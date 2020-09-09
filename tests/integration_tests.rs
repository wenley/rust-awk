extern crate rust_awk;

// use std::fs::read_to_string;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str::from_utf8;

fn run_command_with_input(command: &mut Command, input: &str) -> String {
    let mut piped = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn");
    {
        let stdin = piped.stdin.as_mut().expect("Failed to open StdIn");
        stdin.write(input.as_bytes()).expect("Failed to write");
    }

    let raw_output = piped
        .wait_with_output()
        .expect("Failed to read StdOut")
        .stdout;
    from_utf8(&raw_output)
        .expect("Failed to deserialize")
        .to_string()
}

fn check_program_with_input(program_string: &str, input: &str) {
    let rust_output = run_command_with_input(
        Command::new("cargo").args(&["run", "--bin", "rust-awk", program_string]),
        input,
    );
    let reference_output =
        run_command_with_input(Command::new("awk").args(&[program_string]), input);

    assert_eq!(rust_output, reference_output);
}

fn check_program_path_with_input_path(program_path: &str, input_path: &str) {
    let rust_output = run_command_with_input(
        Command::new("cargo").args(&[
            "run",
            "--bin",
            "rust-awk",
            "--",
            "-f",
            program_path,
            input_path,
        ]),
        "",
    );
    let reference_output = run_command_with_input(
        Command::new("awk").args(&["-f", program_path, input_path]),
        "",
    );

    assert_eq!(rust_output, reference_output);
}

#[test]
fn first_test() {
    let program_string = r#"{ print($0); }"#;
    let input = r#"foo
    bar
    baz"#;
    check_program_with_input(program_string, input);
}

#[test]
fn accepts_dash_f_to_specify_file() {
    let output = run_command_with_input(
        Command::new("cargo").args(&[
            "run",
            "--bin",
            "rust-awk",
            "--",
            "-f",
            "tests/test_cases/echo_program/program.awk",
        ]),
        "hello",
    );
    assert_eq!(output, "hello\n");
}

#[test]
fn allows_multiple_input_files() {}

#[test]
fn all_integration_tests() {
    for directory in std::fs::read_dir("tests/test_cases/").unwrap() {
        let directory_path = directory.unwrap().path().to_str().unwrap().to_string();
        let program_path = format!("{}/program.awk", directory_path);
        for file in std::fs::read_dir(directory_path).unwrap() {
            let input_path = file.unwrap().path().to_str().unwrap().to_string();
            check_program_path_with_input_path(&program_path, &input_path);
        }
    }
}
