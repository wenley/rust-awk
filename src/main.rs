use std::env;
use std::fs::File;
use std::io::{stdin, BufReader};

extern crate rust_awk;

fn main() {
    // Don't need the program name
    let args: Vec<String> = env::args().skip(1).collect();
    let (mut run, input_file_paths) = rust_awk::start_run(args);

    run.output_for_begin_items()
        .iter()
        .for_each(|line| println!("{}", line));

    if input_file_paths.len() == 0 {
        for line in run.process_file(&mut stdin()) {
            println!("{}", line);
        }
    } else {
        for filepath in input_file_paths.iter() {
            process_file(&mut run, &filepath);
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
    let mut reader = BufReader::new(file);
    for line in run.process_file(&mut reader) {
        println!("{}", line);
    }
}
