extern crate rand;

use std::env::temp_dir;
use std::ffi::OsString;
use std::fs::{File, read_dir, remove_dir_all};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use rand::random;

const BUFFER_SIZE: usize = 4096;

struct TempDir {
    path: PathBuf,
    string: OsString,
}

impl TempDir {
    fn new() -> TempDir {
        let path = temp_dir().join(format!("test{}", random::<u32>()));
        let string = path.clone().into_os_string();
        TempDir {
            path: path,
            string: string,
        }
    }

    fn as_str(&self) -> &str {
        self.string.to_str().unwrap()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        remove_dir_all(&self.path).unwrap();
    }
}

fn assert_file(expected_file_name: String, actual_file_name: String) {
    assert!(expected_file_name != actual_file_name);
    let mut expected_file = File::open(expected_file_name).unwrap();
    let mut actual_file = File::open(actual_file_name).unwrap();
    let mut buffer1 = [0; BUFFER_SIZE];
    let mut buffer2 = [0; BUFFER_SIZE];

    while let (Ok(size1), Ok(size2)) = (expected_file.read(&mut buffer1), actual_file.read(&mut buffer2)) {
        assert_eq!(size1, size2);
        if size1 == 0 {
            break;
        }

        assert_eq!(&buffer1[..], &buffer2[..]);
    }
}

fn list(filename: &str) {
    let name = format!("tests/{}", filename);
    let mut process = Command::new("target/debug/uncbv");
    process.args(&["list", &format!("{}.cbv", &name)]);
    let output = String::from_utf8(process.output().unwrap().stdout).unwrap();
    let mut output_files: Vec<_> = output.split("\n").collect();
    output_files.pop();
    output_files.sort();

    let mut expected_files: Vec<_> = read_dir(&name)
        .unwrap()
        .map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap())
        .collect();
    expected_files.sort();
    assert_eq!(expected_files, output_files);
}

#[test]
fn list_files() {
    list("twic1134");
    list("small");
}

fn extract(filename: &str) {
    let temp_dir = TempDir::new();
    let dir_name = temp_dir.as_str();
    let name = format!("tests/{}", filename);
    let mut process = Command::new("target/debug/uncbv");
    process.args(&["extract", &format!("{}.cbv", name), "-o", dir_name])
        .status()
        .unwrap();

    let expected_files: Vec<_> = read_dir(&name)
        .unwrap()
        .map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap())
        .collect();

    for file in expected_files {
        assert_file(format!("{}/{}", name, file), format!("{}/{}", dir_name, file));
    }
}

#[test]
fn extract_files() {
    extract("twic1134");
    extract("small");
}
