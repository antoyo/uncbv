use std::fs::read_dir;
use std::process::Command;

#[test]
fn list_files() {
    let name = "tests/twic1134";
    let mut process = Command::new("target/debug/uncbv");
    process.args(&["list", &format!("{}.cbv", name)]);
    let output = String::from_utf8(process.output().unwrap().stdout).unwrap();
    let mut output_files: Vec<_> = output.split("\n").collect();
    output_files.pop();
    output_files.sort();

    let mut expected_files: Vec<_> = read_dir(name)
        .unwrap()
        .map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap())
        .collect();
    expected_files.sort();
    assert_eq!(expected_files, output_files);
}
