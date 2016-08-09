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
 * TODO: Better error handling.
 * TODO: Decompress the files concurrently.
 * TODO: Write in a channel instead of in the files directly (perhaps there is a better way).
 * TODO: Decompress as a vector of Byte | RunLength | BackwardReference (perhaps there is a better
 * way).
 * TODO: check whether the output files already exists. If so, ask to overwrite.
 * TODO: create a crate for the cbv parser.
 * TODO: add the coverage badge and update the travis script to use coveralls (since travis-cargo
 * does not work).
 */

extern crate docopt;
extern crate encoding;
extern crate huffman;
extern crate memmap;
#[macro_use]
extern crate nom;
extern crate rustc_serialize;

mod archive;
#[macro_use]
mod macros;
mod cbv;

use docopt::Docopt;

use archive::{extract, get_file_list};

const USAGE: &'static str = "
CBV unarchiver.

Usage:
    uncbv (l | list) <filename>
    uncbv (x | extract) <filename> [(-o <output> | --output=<output>)]
    uncbv (-h | --help)
    uncbv (-V | --version)

Options:
    -h --help               Show this help.
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
    flag_output: Option<String>,
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
        let output = args.flag_output.unwrap_or(".".to_string());
        parse_or_show_error!(extract, filename, &output);
    }
}
