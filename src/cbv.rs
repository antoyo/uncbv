/*
 * Copyright (C) 2016  Boucher, Antoni <bouanto@zoho.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! CBV file format parser.

use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::Path;

use encoding::{DecoderTrap, Encoding};
use encoding::all::ISO_8859_1;
use nom::{le_i32, le_u8, le_u16, non_empty};
use nom::IResult::{self, Done};

/// Block compression flags.
struct CompressionFlags {
    compressed: bool,
    huffman_encoded: bool
}

/// File meta-data.
#[derive(Debug)]
pub struct FileMetaData {
    pub compressed_size: i32,
    pub decompressed_size: i32,
    pub filename: String,
}

impl FileMetaData {
    fn new(filename: String, compressed_size: i32, decompressed_size: i32) -> FileMetaData {
        FileMetaData {
            compressed_size: compressed_size,
            decompressed_size: decompressed_size,
            filename: filename.to_string(),
        }
    }
}

/// CBV archive header.
#[derive(Debug)]
struct Header {
    file_count: usize,
    filename_len: u8,
}

impl Header {
    fn new(file_count: u16, filename_len: u8) -> Header {
        Header {
            file_count: file_count as usize,
            filename_len: filename_len,
        }
    }
}

/// Parse a CBV file header.
named!(header <Header>,
    chain!
        ( tag!(&[0x08, 0x00]) // CBV magic number.
        ~ file_count: le_u16
        ~ filename_len: le_u8
        ~ take!(3) // NOTE: unknown bytes.
        , || {
            Header::new(file_count, filename_len)
        })
);

/// Parse a null-terminated String as a filename.
named!(filename <String>,
    map!(
        flat_map!(
            take!(132),
            take_while!(is_not_zero)
        ),
        bytes_to_filename
    )
);

/// Parse the file metadata (name and sizes).
named!(file_metadata <FileMetaData>,
    chain!
        ( filename: filename
        ~ compressed_size: le_i32
        ~ decompressed_size: le_i32
        , || FileMetaData::new(filename, compressed_size, decompressed_size)
        )
);

/// Parse the file list.
named_args!(file_list(header: Header) < Vec<FileMetaData> >,
    count!(
        flat_map!(
            take!(header.filename_len),
            file_metadata
        ),
        header.file_count
    )
);

/// Parse only the filenames from the archive.
named!(pub extract_file_list < Vec<FileMetaData> >,
    chain!
        ( header: header
        ~ file_list: apply!(file_list, header)
        , || file_list
        )
);

/// Parse a compressed block.
named_args!(block<'a>(file: &FileMetaData, output_dir: &str) <()>,
    chain!
        ( block_size: le_u16
        ~ le_u16 // NOTE: unknown bytes.
        ~ flat_map!(
              take!(block_size),
              apply!(extract_block, file, output_dir)
          )
        , || ()
        )
);

/// Parse the compression flag.
named!(compression_flag <CompressionFlags>,
    alt!( tag!(&[0b00]) => { |_| CompressionFlags { compressed: false, huffman_encoded: false } }
        | tag!(&[0b01]) => { |_| CompressionFlags { compressed: true, huffman_encoded: false } }
        | tag!(&[0b10]) => { |_| CompressionFlags { compressed: false, huffman_encoded: true } }
        | tag!(&[0b11]) => { |_| CompressionFlags { compressed: true, huffman_encoded: true } }
        )
);

/// Decompress a block.
fn decompress_block(mut input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let mut result = vec![];

    'block_loop:
    while !input.is_empty() {
        let (new_input, mut code_bytes) = try_parse!(input, le_u16);
        input = new_input;

        for _ in 0 .. 16 {
            let coded = (code_bytes & 0x8000) != 0;
            if coded {
                // TODO: write a wrapper to a slice where the Index operator uses get_unchecked() in
                // release and [] in debug.
                let current_byte = input[0] as usize;
                let high = current_byte >> 4;
                let low = current_byte & 0xF;
                if high == 0 {
                    // Run-length decoding.
                    let size = low + 3;
                    result.append(&mut vec![input[1]; size]);
                }
                else if high == 1 {
                    // Run-length decoding with bigger size.
                    unimplemented!();
                }
                else {
                    // Copy content already seen in the file (backward reference).
                    // Get the offset and the length.
                    let offset = ((input[1] as usize) << 4) + low + 3;
                    let size =
                        if high == 2 {
                            let size = input[2] + 0x10;
                            input = &input[1..];
                            size as usize
                        }
                        else {
                            high
                        };
                    let current_position = result.len();
                    let start = current_position - offset;
                    let end = start + size;
                    let mut backward_reference = result[start .. end].iter().cloned().collect();
                    result.append(&mut backward_reference);
                }
                input = &input[1..];
            }
            else {
                result.push(input[0]);
            }
            input = &input[1..];
            if input.is_empty() {
                break 'block_loop;
            }
            code_bytes <<= 1;
        }
    }
    Done(input, result)
}

/// Extract, decode and decompress a block.
named_args!(extract_block<'a>(file: &FileMetaData, output_dir: &str) <()>,
    chain!
        ( flag: compression_flag
        ~ result: flat_map!(
              parse_if!(flag.huffman_encoded, huffman),
              parse_if_else!(flag.compressed, decompress_block, |input| {
                  let mut result = vec![];
                  result.extend_from_slice(input);
                  result
              })
          )
        , || {
            create_dir_all(output_dir).unwrap();
            let path = Path::new(output_dir).join(&file.filename);
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();

            file.write_all(&result).unwrap();
        }
        )
);

/// Extract a file from the archive.
named_args!(extract_file<'a>(file: FileMetaData, output_dir: &str) <()>,
    flat_map!(
        take!(file.compressed_size),
        fold_many1!(apply!(block, &file, output_dir), (), |_, _| ())
    )
);

/// Extract the files from the archive.
named_args!(pub extract_files<'a>(output_dir: &str) <()>,
    chain!
        ( files: extract_file_list
        ~ foreach!(files, file => apply!(extract_file, file, output_dir))
        , || ()
        )
);

/// Decode a huffman-encoded block.
named!(huffman <&[u8]>,
    chain!
        ( non_empty
        , || panic!("Huffman decoding unimplemented.")
        )
);

/// Convert the bytes reprensenting the filename into a String, replacing the backslashes by
/// slashes and converting the filename to UTF-8.
fn bytes_to_filename(bytes: &[u8]) -> String {
    let mut string = ISO_8859_1.decode(bytes, DecoderTrap::Strict)
        .unwrap(); // NOTE: The filename should be valid.
    replace_backslash_by_slash(&mut string);
    string
}

/// Check if the byte is different than zero.
fn is_not_zero(byte: u8) -> bool {
    byte != 0
}

/// Replace all the backslashes by slashes in `string`.
fn replace_backslash_by_slash(string: &mut String) {
    let bytes = unsafe { string.as_mut_vec() };
    for byte in bytes {
        if *byte == b'\\' {
            *byte = b'/';
        }
    }
}
