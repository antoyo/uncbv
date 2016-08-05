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

use encoding::{DecoderTrap, Encoding};
use encoding::all::ISO_8859_1;
use nom::{le_u8, le_u16};
use nom::IResult;

/// CBV archive header.
#[derive(Debug)]
struct Header {
    file_count: usize,
    filename_len: u8,
}

impl Header {
    /// Create a new `Header`.
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
        ( tag!(&[0x08, 0x00]) // NOTE: CBV magic number.
        ~ file_count: le_u16
        ~ filename_len: le_u8
        ~ take!(3) // NOTE: unknown bytes.
        , || {
            Header::new(file_count, filename_len)
        })
);

/// Wrapper for the file_list parser function.
macro_rules! file_list {
    ($input:expr, $header:expr) => {
        file_list($input, $header)
    };
}

/// Parse the file list.
fn file_list(input: &[u8], header: Header) -> IResult<&[u8], Vec<String>> {
    /// Convert the bytes reprensenting the filename into a String, replacing the backslashes by
    /// slashes and converting the filename to UTF-8.
    fn convert_filename(bytes: &[u8]) -> String {
        let end_index = bytes.iter()
            .position(|&byte| byte == 0)
            .unwrap_or(bytes.len());

        let mut string = ISO_8859_1.decode(&bytes[..end_index], DecoderTrap::Strict)
            .unwrap();

        replace_backslash_by_slash(&mut string);

        string
    }

    count!(
        input,
        map!(
            take!(header.filename_len),
            convert_filename
        ),
        header.file_count
    )
}

/// Parse only the filenames from the archive.
named!(pub archive_filenames <Vec<String> >,
    chain!
        ( header: header
        ~ file_list: file_list!(header)
        , || file_list
        )
);

/// Replace all the backslashes by slashes in `string`.
fn replace_backslash_by_slash(string: &mut String) {
    let bytes = unsafe { string.as_mut_vec() };
    for byte in bytes {
        if *byte == b'\\' {
            *byte = b'/';
        }
    }
}
