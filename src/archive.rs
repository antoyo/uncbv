//! CBV archive utility functions.

use std::ffi::OsStr;
use std::io::Error;
use std::path::Path;

use memmap::{Mmap, Protection};
use nom::IResult::Done;

use cbv::archive_filenames;

/// Check if the file extension belongs to an encrypted CBV archive (.cbz).
fn is_encrypted_archive(filename: &str) -> bool {
    let path = Path::new(filename);
    path.extension() == Some(OsStr::new("cbz"))
}

/// Extract the filenames from the archive.
pub fn extract_filenames(filename: &str) -> Result<Vec<String>, Error> {
    if is_encrypted_archive(filename) {
        panic!("Encrypted archive is not supported at the moment.");
    }
    else {
        let file = try!(Mmap::open_path(filename, Protection::Read));
        let bytes: &[u8] = unsafe { file.as_slice() };
        match archive_filenames(bytes) {
            Done(_, filenames) => Ok(filenames),
            _ => panic!("Error"), // TODO
        }
    }
}

/// Unarchive and extract a CBV archive.
pub fn unarchive(filename: &str) -> Result<(), Error> {
    if is_encrypted_archive(filename) {
        panic!("Encrypted archive is not supported at the moment.");
    }
    else {
        let file = try!(Mmap::open_path(filename, Protection::Read));
        let bytes: &[u8] = unsafe { file.as_slice() };
        if let Done(_, result) = archive_filenames(bytes) {
            println!("{:?}", result);
        }
    }
    Ok(())
}
