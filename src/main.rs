use std::env;
use std::io;
extern crate rust_awk;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = rust_awk::parse_program("{ print(\"hello\"); }");
    let mut run = rust_awk::start_run(&program);

    println!("Hello, world!");
    println!("args = {:?}", args);

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
                buffer.truncate(n - 1);
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
