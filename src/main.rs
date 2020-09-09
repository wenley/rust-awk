use std::env;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};

extern crate rust_awk;

fn main() {
    // Don't need the program name
    let args: Vec<String> = env::args().skip(1).collect();
    let (mut run, input_file_paths) = rust_awk::start_run(args);

    run.output_for_begin_items()
        .iter()
        .for_each(|line| println!("{}", line));

    if input_file_paths.len() == 0 {
        process_stdin(run);
    } else {
        for filepath in input_file_paths.iter() {
            process_file(&mut run, &filepath);
        }
    }
}

fn process_stdin(mut run: rust_awk::ProgramRun) {
    let stdin = stdin();
    let mut buffer = String::new();
    loop {
        match stdin.read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                if buffer.chars().last().unwrap() == '\n' {
                    buffer.truncate(n - 1);
                }
                for line in run.output_for_line(&buffer) {
                    println!("{}", line);
                }
                buffer.clear();
            }
            Err(error) => {
                eprintln!("Error encountered: {}", error);
                break;
            }
        }
    }
}

fn process_file(run: &mut rust_awk::ProgramRun, path: &str) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error encountered: {}", e);
            return;
        }
    };
    let mut buffer = String::new();
    let mut reader = BufReader::new(file);
    loop {
        match reader.read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                if buffer.chars().last().unwrap() == '\n' {
                    buffer.truncate(n - 1);
                }
                for output_line in run.output_for_line(&buffer) {
                    println!("{}", output_line);
                }
                buffer.clear();
            }
            Err(error) => {
                eprintln!("Error encountered: {}", error);
                break;
            }
        }
    }
}
