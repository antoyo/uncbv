extern crate rand;
extern crate walkdir;

use std::env::temp_dir;
use std::ffi::OsString;
use std::fs::{File, read_dir, remove_dir_all, remove_file};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use rand::random;
use walkdir::WalkDir;

const BUFFER_SIZE: usize = 4096;
const DEFAULT_PASSWORD: &'static str = "password";

#[test]
fn decrypt_files() {
    decrypt("small", DEFAULT_PASSWORD, "small");
    decrypt("small2", "pass", "small");
    decrypt("small3", "my long password", "small");
}

#[test]
fn extract_files() {
    extract("twic1134");
    extract("small");
    decrypt_extract("small", DEFAULT_PASSWORD);
}

#[test]
#[ignore]
fn extract_files2() {
    let (_files_to_decrypt, files_to_extract) = others();
    for filename in files_to_extract {
        extract(&filename);
    }

    /*for filename in files_to_decrypt {
        decrypt_extract(&filename, DEFAULT_PASSWORD);
    }*/
}

#[test]
fn list_files() {
    list("twic1134");
    list("small");
    list_encrypted("small", DEFAULT_PASSWORD);
}

#[test]
#[ignore]
fn list_files2() {
    let (files_to_decrypt, files_to_extract) = others();
    for filename in files_to_extract {
        list(&filename);
    }

    for filename in files_to_decrypt {
        list_encrypted(&filename, DEFAULT_PASSWORD);
    }
}

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

fn decrypt(filename: &str, password: &str, expected_dir: &str) {
    let path = temp_dir().join(format!("{}.cbv", filename));
    let output_file = path.to_str().unwrap();
    let expected_dir = format!("tests/{}", expected_dir);
    let name = format!("tests/{}", filename);
    let mut process = Command::new(uncbv_executable());
    let mut child =
        process.args(&["decrypt", &format!("{}.cbz", name), "-o", &output_file])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped()) // NOTE: hide the password prompt.
            .spawn()
            .unwrap();
    writeln!(child.stdin.as_mut().unwrap(), "{}", password).unwrap();
    child.wait().unwrap();
    extract_decrypted(output_file, &expected_dir);
    remove_file(output_file).unwrap();
}

fn decrypt_extract(filename: &str, password: &str) {
    let temp_dir = TempDir::new();
    let dir_name = temp_dir.as_str();
    let name = format!("tests/{}", filename);
    let mut process = Command::new(uncbv_executable());

    let mut child =
        process.args(&["extract", &format!("{}.cbz", name), "-o", dir_name])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped()) // NOTE: hide the password prompt.
            .spawn()
            .unwrap();

    writeln!(child.stdin.as_mut().unwrap(), "{}", password).unwrap();
    child.wait().unwrap();

    let expected_files = get_file_recursives(&name);

    assert!(expected_files.len() > 1);

    for file in expected_files {
        assert_file(format!("{}/{}", name, file), format!("{}/{}", dir_name, file));
    }
}

fn extract(filename: &str) {
    let temp_dir = TempDir::new();
    let dir_name = temp_dir.as_str();
    let name = format!("tests/{}", filename);
    let mut process = Command::new(uncbv_executable());
    process.args(&["extract", &format!("{}.cbv", name), "-o", dir_name])
        .status()
        .unwrap();

    let expected_files = get_file_recursives(&name);

    assert!(expected_files.len() > 1);

    for file in expected_files {
        assert_file(format!("{}/{}", name, file), format!("{}/{}", dir_name, file));
    }
}

// The parameter filename is an absolute path.
fn extract_decrypted(filename: &str, expected_dir: &str) {
    let temp_dir = TempDir::new();
    let dir_name = temp_dir.as_str();
    let mut process = Command::new(uncbv_executable());

    process.args(&["extract", &filename.to_string(), "-o", dir_name])
        .status()
        .unwrap();

    let expected_files = get_file_recursives(expected_dir);

    assert!(expected_files.len() > 1);

    for file in expected_files {
        assert_file(format!("{}/{}", expected_dir, file), format!("{}/{}", dir_name, file));
    }
}

fn get_file_recursives(directory: &str) -> Vec<String> {
    let mut expected_files: Vec<String> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|file| {
            let file = file.unwrap();
            if file.file_type().is_dir() {
                None
            }
            else {
                let path = file.path().strip_prefix(directory).unwrap();
                Some(path.to_str().unwrap().to_string())
            }
        })
        .collect();
    expected_files.sort();
    expected_files
}

fn list(filename: &str) {
    let name = format!("tests/{}", filename);
    let mut process = Command::new(uncbv_executable());
    process.args(&["list", &format!("{}.cbv", &name)]);
    let output = String::from_utf8(process.output().unwrap().stdout).unwrap();
    let mut output_files: Vec<_> = output.split("\n").collect();
    output_files.pop();
    output_files.sort();

    let expected_files = get_file_recursives(&name);
    assert!(expected_files.len() > 1);
    assert_eq!(expected_files, output_files);
}

fn list_encrypted(filename: &str, password: &str) {
    let name = format!("tests/{}", filename);
    let mut process = Command::new(uncbv_executable());
    let filename = format!("{}.cbz", &name);

    let mut child =
        process.args(&["list", &filename])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

    writeln!(child.stdin.as_mut().unwrap(), "{}", password).unwrap();
    let output = String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap();

    let mut output_files: Vec<_> = output.split("\n").collect();
    output_files.remove(0); // NOTE: Remove the "Password:" line.
    output_files.pop();
    output_files.sort();

    let expected_files = get_file_recursives(&name);
    assert!(expected_files.len() > 1);
    assert_eq!(expected_files, output_files);
}

fn others() -> (Vec<String>, Vec<String>) {
    let mut files_to_decrypt = vec![];
    let mut files_to_extract = vec![];
    for file in read_dir("tests/others").unwrap() {
        let mut path = file.unwrap().path();
        let mut to_decrypt = false;
        let mut to_extract = false;
        if let Some(extension) = path.extension() {
            let extension = extension.to_str().unwrap();
            match extension {
                "cbv" => to_extract = true,
                "cbz" => to_decrypt = true,
                _ => (),
            }
        }

        path.set_extension("");
        let path = path.strip_prefix("tests").unwrap();
        let filename = path.to_str().unwrap();
        if to_extract {
            files_to_extract.push(filename.to_string());
        }
        else if to_decrypt {
            files_to_decrypt.push(filename.to_string());
        }
    }

    (files_to_decrypt, files_to_extract)
}

#[cfg(debug_assertions)]
fn uncbv_executable() -> String {
    "target/debug/uncbv".to_string()
}

#[cfg(not(debug_assertions))]
fn uncbv_executable() -> String {
    "target/release/uncbv".to_string()
}
