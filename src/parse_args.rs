use std::collections::HashMap;

pub(crate) struct Args {
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

pub(crate) fn parse_args(args: Vec<String>) -> Args {
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
                match &arg[..] {
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
                let mut var_pair: Vec<&str> = arg.splitn(2, "=").collect();
                let value = var_pair.pop().unwrap().to_string();
                let var_name = var_pair.pop().unwrap().to_string();

                parsed_args.variables.insert(var_name, value);
                state = ParsingState::Neutral;
            }
            ParsingState::ExpectProgramFileName => {
                // TODO: Parse from filepath
                let program_string = "aoeu".to_string();
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
