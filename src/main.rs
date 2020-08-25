use std::env;
use std::io;
extern crate rust_awk;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = rust_awk::parse::parse_program("{ print(\"hello\"); }");
    let mut run = rust_awk::start_run(&program);

    println!("Hello, world!");
    println!("args = {:?}", args);

    run.execute_begin();

    let stdin = io::stdin();

    let mut buffer = String::new();
    loop {
        match stdin.read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                buffer.truncate(n - 1);
                let fields = run.split(&buffer);
                let record = rust_awk::basic_types::Record {
                    full_line: &buffer,
                    fields: &fields,
                };
                for line in run.output_for_line(&record) {
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
