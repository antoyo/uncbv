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

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions, create_dir_all};
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

/// Ask to override a file.
fn ask_override_file(path: &Path) -> bool {
    if path.exists() {
        let path = path.to_str().unwrap();
        println!("The file {} already exists. Do you wish to override it? [y/N]", path);
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).unwrap();
        answer.chars().next().unwrap().to_lowercase().collect::<String>() == "y"
    }
    else {
        true
    }
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
/// Returns whether the archive has been decrypted or not.
pub fn decrypt_archive(filename: &str, output: Option<String>, no_confirm: bool) -> bool {
    let output = output.unwrap_or_else(|| {
        let mut path = PathBuf::from(filename);
        path.set_extension("cbv");
        path.into_os_string().into_string().unwrap()
    });

    let override_file = no_confirm || ask_override_file(Path::new(&output));

    if override_file {
        let password = ask_password();
        let mut input_file = File::open(filename).unwrap();
        {
            let output_dir = Path::new(&output).parent().unwrap();
            create_dir_all(output_dir).unwrap();
        }
        let mut file = File::create(output).unwrap();
        decrypt(&mut input_file, &password, &mut file).unwrap();
    }
    override_file
}

/// Decrypt, unarchive and decompress the files from a CBV archive.
pub fn extract(filename: &str, output_dir: &str, no_confirm: bool) -> Result<(), Error> {
    let output_path = Path::new(output_dir);
    let filename =
        if is_encrypted_archive(filename) {
            let mut path = PathBuf::from(filename);
            path.set_extension("cbv");
            let new_filename = path.file_name().unwrap().to_str().unwrap();
            try!(create_dir_all(output_dir));
            let output_file_path = output_path.join(new_filename);
            let output_file = output_file_path.into_os_string().into_string().unwrap();;

            if !decrypt_archive(filename, Some(output_file.clone()), no_confirm) {
                return Ok(());
            }
            output_file
        }
        else {
            filename.to_string()
        };

    let file = try!(Mmap::open_path(filename, Protection::Read));
    let bytes: &[u8] = unsafe { file.as_slice() };
    let file_list = unwrap_or_error!(extract_file_list(bytes));

    let first_file_path = output_path.join(&file_list[0].filename);
    let override_file = no_confirm || ask_override_file(first_file_path.as_path());

    if override_file {
        try!(init_output(&file_list, output_dir));
        // TODO: do not parse again the file list.
        unwrap_or_error!(extract_files(bytes, output_dir))
    }

    Ok(())
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

/// Create the required directories and truncate the existing files.
fn init_output(file_list: &[FileMetaData], output_dir: &str) -> Result<(), Error> {
    let output_path = Path::new(output_dir);
    let mut directories = HashSet::new();
    let mut files = vec![];

    for file in file_list {
        let file_path = output_path.join(&file.filename);
        let directory = file_path.parent().unwrap().to_path_buf();
        directories.insert(directory);
        files.push(file_path);
    }

    for directory in directories {
        try!(create_dir_all(directory));
    }

    for file in files {
        try!(OpenOptions::new()
             .create(true)
             .write(true)
             .truncate(true)
             .open(&file));
    }

    Ok(())
}

/// Check if the file extension belongs to an encrypted CBV archive (.cbz).
fn is_encrypted_archive(filename: &str) -> bool {
    let path = Path::new(filename);
    path.extension() == Some(OsStr::new("cbz"))
}
