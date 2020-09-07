use std::env;
use std::io;

extern crate rust_awk;

fn main() {
    // Don't need the program name
    let args: Vec<String> = env::args().skip(1).collect();

    let parsed_args = rust_awk::parse_args::parse_args(args);

    let program = rust_awk::parse_program(&parsed_args.program_string.clone().unwrap());
    let mut run = rust_awk::start_run(&program);
    run.apply_args(&parsed_args);

    run.output_for_begin_items()
        .iter()
        .for_each(|line| println!("{}", line));

    let stdin = io::stdin();

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
            }
        }
    }
}
