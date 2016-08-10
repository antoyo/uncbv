extern crate rand;

use std::env::temp_dir;
use std::ffi::OsString;
use std::fs::{File, read_dir, remove_dir_all, remove_file};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

fn list(filename: &str, password: Option<&str>) {
    let has_password = password.is_some();
    let name = format!("tests/{}", filename);
    let mut process = Command::new("target/debug/uncbv");
    let filename =
        if has_password {
            format!("{}.cbz", &name)
        }
        else {
            format!("{}.cbv", &name)
        };

    let mut child =
        process.args(&["list", &filename])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

    if let Some(password) = password {
        writeln!(child.stdin.as_mut().unwrap(), "{}", password).unwrap();
    }
    child.wait().unwrap();

    let mut output = String::new();
    child.stdout.unwrap().read_to_string(&mut output).unwrap();
    let mut output_files: Vec<_> = output.split("\n").collect();
    if has_password {
        output_files.remove(0); // NOTE: Remove the "Password:" line.
    }
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
    list("twic1134", None);
    list("small", Some("password"));
}

fn extract(filename: &str, expected_file: Option<&str>, password: Option<&str>) {
    let temp_dir = TempDir::new();
    let dir_name = temp_dir.as_str();
    let name = format!("tests/{}", filename);
    let mut process = Command::new("target/debug/uncbv");

    let input_file =
        if Path::new(filename).is_absolute() {
            filename.to_string()
        }
        else {
            if password.is_some() {
                format!("{}.cbz", name)
            }
            else {
                format!("{}.cbv", name)
            }
        };

    let mut child =
        process.args(&["extract", &input_file, "-o", dir_name])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

    if let Some(password) = password {
        writeln!(child.stdin.as_mut().unwrap(), "{}", password).unwrap();
    }
    child.wait().unwrap();

    let name = expected_file.unwrap_or(&name);
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
    extract("twic1134", None, None);
    extract("small", None, None);
}

fn decrypt(filename: &str, password: &str, output_dir: &str) {
    let path = temp_dir().join(format!("{}.cbv", filename));
    let output_file = path.to_str().unwrap();
    let output_dir = format!("tests/{}", output_dir);
    let name = format!("tests/{}", filename);
    let mut process = Command::new("target/debug/uncbv");
    let mut child =
        process.args(&["decrypt", &format!("{}.cbz", name), "-o", &output_file])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
    writeln!(child.stdin.as_mut().unwrap(), "{}", password).unwrap();
    child.wait().unwrap();
    extract(output_file, Some(&output_dir), None);
    remove_file(output_file).unwrap();
}

fn decrypt_extract(filename: &str, password: &str) {
    extract(filename, None, Some(password));
}

#[test]
fn decrypt_files() {
    decrypt("small", "password", "small");
    decrypt("small2", "pass", "small");
    decrypt("small3", "my long password", "small");
    decrypt_extract("small", "password");
}
