use std::env;
use std::io;
use std::collections::HashMap;

extern crate rust_awk;

struct Args {
    field_separator: String,
    variables: HashMap<String, String>,
    program_string: Option<String>,
    filepaths_to_parse: Vec<String>,
}

#[derive(PartialEq, Debug)]
enum ParsingState {
    Neutral,
    ExpectFieldSeparator,
    ExpectVariable,
    ExpectProgramFileName,
}

fn parse_args(args: Vec<String>) -> Args {
    let mut parsed_args = Args {
        field_separator: " ".to_string(),
        variables: HashMap::new(),
        program_string: None,
        filepaths_to_parse: vec![],
    };
    let mut state = ParsingState::Neutral;
    for arg in args {
        match state {
            ParsingState::Neutral => {
                match arg {
                    "-F" => state = ParsingState::ExpectFieldSeparator,
                    "-f" => state = ParsingState::ExpectProgramFileName,
                    "-v" => state = ParsingState::ExpectVariable,
                    _ => {
                        if parsed_args.program_string.is_none() {
                            parsed_args.program_string = Some(arg);
                        } else {
                            parsed_args.filepaths_to_parse.push(arg);
                        }
                    }
                }
            }
            ParsingState::ExpectFieldSeparator => {
                parsed_args.field_separator = arg;
                state = ParsingState::Neutral;
            }
            ParsingState::ExpectVariable => {
                let mut var_pair = arg.splitn(2, "=").collect();
                let value = var_pair.pop();
                let var_name = var_pair.pop();

                parsed_args.variables.insert(var_name, value);
                state = ParsingState::Neutral;
            }
            ParsingState::ExpectProgramFileName => {
                // TODO: Parse from filepath
                let program_string = "aoeu";
                parsed_args.program_string = Some(program_string);
                state = ParsingState::Neutral;
            }
        }
    }

    if state != ParsingState::Neutral {
        panic!("Did not finish parsing! Still in {:?}", state);
    }
    if parsed_args.program_string.is_none() {
        panic!("No program provided!");
    }

    parsed_args
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let program = rust_awk::parse_program(&args[1]);
    let mut run = rust_awk::start_run(&program);

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
