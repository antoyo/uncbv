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
use std::fs::{File, create_dir_all};
use std::io::{self, Error, ErrorKind, Read};
use std::path::{Path, PathBuf};

use memmap::{Mmap, Protection};
use nom::IResult::{self, Done, Incomplete};

use cbv::{self, FileMetaData, extract_file_list, extract_files, file_list};
use decrypt::decrypt;

const HEADER_SIZE: usize = 8;

/// Unwrap a Done or return an error.
macro_rules! unwrap_or_error {
    ($val:expr, $message:expr) => {
        match $val {
            Done(_, filenames) => filenames,
            IResult::Error(_) | Incomplete(_) => return Err(Error::new(ErrorKind::InvalidInput, $message)),
        }
    };
    ($val:expr) => {
        match $val {
            Done(_, filenames) => filenames,
            IResult::Error(_) | Incomplete(_) => return Err(Error::new(ErrorKind::InvalidInput, "Not a CBV archive")),
        }
    };
}

/// Ask for the password.
fn ask_password() -> String {
    let mut password = String::new();
    println!("Password:");
    io::stdin().read_line(&mut password).unwrap();
    password.pop();
    password
}

/// Ask for the password and decrypt the archive.
pub fn decrypt_archive(filename: &str, output: Option<String>) {
    let password = ask_password();
    let output = output.unwrap_or_else(|| {
        let mut path = PathBuf::from(filename);
        path.set_extension("cbv");
        path.into_os_string().into_string().unwrap()
    });

    let mut input_file = File::open(filename).unwrap();
    let mut file = File::create(output).unwrap();
    decrypt(&mut input_file, &password, &mut file).unwrap();
}

/// Decrypt, unarchive and decompress the files from a CBV archive.
pub fn extract(filename: &str, output_dir: &str) -> Result<(), Error> {
    let filename =
        if is_encrypted_archive(filename) {
            let mut path = PathBuf::from(filename);
            path.set_extension("cbv");
            let new_filename = path.file_name().unwrap().to_str().unwrap();
            try!(create_dir_all(output_dir));
            let output_path = Path::new(output_dir).join(new_filename);
            let output_file = output_path.into_os_string().into_string().unwrap();;

            decrypt_archive(filename, Some(output_file.clone()));
            output_file
        }
        else {
            filename.to_string()
        };

    let file = try!(Mmap::open_path(filename, Protection::Read));
    let bytes: &[u8] = unsafe { file.as_slice() };
    Ok(unwrap_or_error!(extract_files(bytes, output_dir)))
}

/// Extract the filenames from the archive.
pub fn get_file_list(filename: &str) -> Result<Vec<FileMetaData>, Error> {
    if is_encrypted_archive(filename) {
        let password = ask_password();
        let mut cbv_output = vec![];
        let mut buffer = [0; HEADER_SIZE];
        let mut file = try!(File::open(filename));

        try!(file.read(&mut buffer[..HEADER_SIZE]));
        try!(decrypt(buffer.as_ref(), &password, &mut cbv_output));

        let header = unwrap_or_error!(cbv::header(&cbv_output), "Wrong password");
        cbv_output.clear();

        let file_list_len = header.total_size();
        let mut buffer = vec![0; file_list_len];
        try!(file.read_exact(&mut buffer));
        try!(decrypt(buffer.as_slice(), &password, &mut cbv_output));

        let result = file_list(&cbv_output, header);

        Ok(unwrap_or_error!(result))
    }
    else {
        let file = try!(Mmap::open_path(filename, Protection::Read));
        let bytes: &[u8] = unsafe { file.as_slice() };
        Ok(unwrap_or_error!(extract_file_list(bytes)))
    }
}

/// Check if the file extension belongs to an encrypted CBV archive (.cbz).
fn is_encrypted_archive(filename: &str) -> bool {
    let path = Path::new(filename);
    path.extension() == Some(OsStr::new("cbz"))
}
