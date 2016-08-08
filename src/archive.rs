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

//! CBV archive utility functions.

use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::path::Path;

use memmap::{Mmap, Protection};
use nom::IResult::{self, Done, Incomplete};

use cbv::{FileMetaData, extract_file_list, extract_files};

/// Unwrap a Done or return an error.
macro_rules! unwrap_or_error {
    ($val:expr) => {
        match $val {
            Done(_, filenames) => filenames,
            IResult::Error(_) | Incomplete(_) => return Err(Error::new(ErrorKind::InvalidInput, "Not a CBV archive")),
        }
    };
}

/// Check if the file extension belongs to an encrypted CBV archive (.cbz).
fn is_encrypted_archive(filename: &str) -> bool {
    let path = Path::new(filename);
    path.extension() == Some(OsStr::new("cbz"))
}

/// Extract the filenames from the archive.
pub fn get_file_list(filename: &str) -> Result<Vec<FileMetaData>, Error> {
    if is_encrypted_archive(filename) {
        panic!("Encrypted archive is not supported at the moment.");
    }
    else {
        let file = try!(Mmap::open_path(filename, Protection::Read));
        let bytes: &[u8] = unsafe { file.as_slice() };
        Ok(unwrap_or_error!(extract_file_list(bytes)))
    }
}

/// Decrypt, unarchive and decompress the files from a CBV archive.
pub fn extract(filename: &str, output_dir: &str) -> Result<(), Error> {
    if is_encrypted_archive(filename) {
        panic!("Encrypted archive is not supported at the moment.");
    }
    else {
        let file = try!(Mmap::open_path(filename, Protection::Read));
        let bytes: &[u8] = unsafe { file.as_slice() };
        unwrap_or_error!(extract_files(bytes, output_dir))
    }

    Ok(())
}
