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
 * TODO: upload on crates.io.
 * TODO: add tests.
 * TODO: add the build, coverage and crates.io badges.
 */

extern crate docopt;
extern crate encoding;
extern crate memmap;
#[macro_use]
extern crate nom;
extern crate rustc_serialize;

mod archive;
mod cbv;

use docopt::Docopt;

use archive::extract_filenames;

const USAGE: &'static str = "
CBV unarchiver.

Usage:
    uncbv (l | list) <filename>
    uncbv (-h | --help)
    uncbv (-V | --version)

Options:
    -h --help       Show this help.
    -V --version    Show the version of uncbv.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_filename: String,
    cmd_list: bool,
}

fn main() {
    let version = format!("{}, version: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let args: Args = Docopt::new(USAGE)
        .and_then(|decoder| decoder.version(Some(version)).decode())
        .unwrap_or_else(|error| error.exit());
    if args.cmd_list {
        match extract_filenames(&args.arg_filename) {
            Ok(files) => {
                for file in files {
                    println!("{}", file);
                }
            },
            Err(error) => println!("{:?}", error),
        }
    }
}
