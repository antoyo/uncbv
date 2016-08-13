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

/*
 * FIXME: encrypted MegaDatabase does not extract (the bytes are incorrect starting at offset
 * 0x2AB50000).
 * FIXME: panic when decrypting into a non-existing directory.
 * FIXME: error when extracting an encrypted database to an existing decrypted database file
 * (instead of overriding).
 * TODO: This software consumes too much memory (because of mmaping the file).
 * TODO: Better error handling.
 * TODO: Decompress the files concurrently.
 * TODO: Add a state representing the result in the decompressor: (&[u8], Vec<u8>) instead of &[u8].
 * TODO: Write in a channel instead of in the files directly (perhaps there is a better way).
 * TODO: Decompress as a vector of Byte | RunLength | BackwardReference (perhaps there is a better
 * way).
 * TODO: Create a crate for the cbv parser (Add the coverage badge and update the travis script to
 * use coveralls (since travis-cargo does not work)).
 */

extern crate des;
extern crate docopt;
extern crate encoding;
extern crate huffman;
extern crate memmap;
#[macro_use]
extern crate nom;
extern crate rustc_serialize;

mod archive;
mod decrypt;
#[macro_use]
mod macros;
mod cbv;

use std::path::Path;

use docopt::Docopt;

use archive::{decrypt_archive, extract, get_file_list};

const USAGE: &'static str = "
CBV unarchiver.

Usage:
    uncbv (l | list) <filename>
    uncbv (x | extract) <filename> [(--output=<output> | --create-dir)] [--no-confirm]
    uncbv (d | decrypt) <filename> [--output=<output>] [--no-confirm]
    uncbv (-h | --help)
    uncbv (-V | --version)

Options:
    -c --create-dir         Extract in a new directory (uncbv extract <filename>.cbv -c is equivalent to uncbv extract <filename>.cbv -o <filename>).
    -h --help               Show this help.
    --no-confirm            Do not ask for any confirmation before overriding.
    -o --output <output>    Set output directory.
    -V --version            Show the version of uncbv.
";

/// Unwrap the result or show the error and return from the function.
macro_rules! parse_or_show_error {
    ($parser:expr, $filename:expr $(, $args:expr )*) => {
        match $parser($filename $(, $args )*) {
            Ok(result) => {
                result
            },
            Err(error) => {
                println!("{}: {}", $filename, error);
                return;
            },
        }
    };
}

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_filename: String,
    flag_create_dir: bool,
    flag_no_confirm: bool,
    flag_output: Option<String>,
    cmd_d: bool,
    cmd_decrypt: bool,
    cmd_extract: bool,
    cmd_l: bool,
    cmd_list: bool,
    cmd_x: bool,
}

fn main() {
    let version = format!("{}, version: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let args: Args = Docopt::new(USAGE)
        .and_then(|decoder| decoder.version(Some(version)).decode())
        .unwrap_or_else(|error| error.exit());
    let filename = &args.arg_filename;
    if args.cmd_list || args.cmd_l {
        let files = parse_or_show_error!(get_file_list, filename);
        for file in files {
            println!("{}", file.filename);
        }
    }
    else if args.cmd_extract || args.cmd_x {
        let output =
            if args.flag_create_dir {
                let path = Path::new(filename);
                path.file_stem().unwrap().to_str().unwrap().to_string()
            }
            else {
                args.flag_output.unwrap_or_else(|| ".".to_string())
            };
        parse_or_show_error!(extract, filename, &output, args.flag_no_confirm);
    }
    else if args.cmd_decrypt || args.cmd_d {
        decrypt_archive(filename, args.flag_output, args.flag_no_confirm);
    }
}
