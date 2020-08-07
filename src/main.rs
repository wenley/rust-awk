
use std::env;
use std::io;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Hello, world!");
    println!("args = {:?}", args);

    let stdin = io::stdin();

    let mut buffer = String::new();
    loop {
        match stdin.read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                buffer.truncate(n - 1);
                println!("{}", buffer);
                buffer.clear();
            }
            Err(error) => {
                eprintln!("Error encountered: {}", error);
            }
        }
    }
}
