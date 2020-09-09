use std::env;
use std::io::{stdin};

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
