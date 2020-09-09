use std::collections::HashMap;

pub struct Args {
    pub(crate) field_separator: String,
    pub(crate) variables: HashMap<String, String>,
    pub(crate) filepaths_to_parse: Vec<String>,
}

#[derive(PartialEq, Debug)]
enum ParsingState {
    Neutral,
    ExpectFieldSeparator,
    ExpectVariable,
    ExpectProgramFileName,
}

pub fn parse_args(args: Vec<String>) -> (String, Args) {
    let mut parsed_args = Args {
        field_separator: " ".to_string(),
        variables: HashMap::new(),
        filepaths_to_parse: vec![],
    };
    let mut program_string = None;
    let mut state = ParsingState::Neutral;
    for arg in args {
        match state {
            ParsingState::Neutral => match &arg[..] {
                "-F" => state = ParsingState::ExpectFieldSeparator,
                "-f" => state = ParsingState::ExpectProgramFileName,
                "-v" => state = ParsingState::ExpectVariable,
                _ => {
                    if program_string.is_none() {
                        program_string = Some(arg);
                    } else {
                        parsed_args.filepaths_to_parse.push(arg);
                    }
                }
            },
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
                let string = std::fs::read_to_string(arg).unwrap();
                program_string = Some(string);
                state = ParsingState::Neutral;
            }
        }
    }

    if state != ParsingState::Neutral {
        panic!("Did not finish parsing! Still in {:?}", state);
    }
    if program_string.is_none() {
        panic!("No program provided!");
    }

    (program_string.unwrap(), parsed_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_program_string() -> &'static str {
        "{ print($0); }"
    }

    fn stringify<'a>(args: Vec<&'a str>) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    #[should_panic]
    fn without_any_args() {
        parse_args(vec![]);
    }

    #[test]
    fn with_just_a_string() {
        let program = basic_program_string();
        let (program_string, _) = parse_args(stringify(vec![program]));
        assert_eq!(program_string, program);
    }

    #[test]
    fn with_multiple_variables() {
        let (_, args) = parse_args(stringify(vec!["-v", "a=b=c", basic_program_string()]));
        assert_eq!(args.variables.len(), 1);
        assert_eq!(args.variables.get("a").unwrap(), "b=c");
    }

    #[test]
    fn with_field_separator() {
        let (_, args) = parse_args(stringify(vec!["-F", "abc", basic_program_string()]));
        assert_eq!(args.field_separator, "abc");
    }

    #[test]
    fn with_separator_between_variables() {
        let (_, args) = parse_args(stringify(vec![
            "-v",
            "foo=123",
            "-F",
            "abc",
            "-v",
            "bar=456",
            basic_program_string(),
        ]));
        assert_eq!(args.field_separator, "abc");
        assert_eq!(args.variables.len(), 2);
        assert_eq!(args.variables.get("foo").unwrap(), "123");
        assert_eq!(args.variables.get("bar").unwrap(), "456");
    }

    #[test]
    fn with_specified_program_file() {}

    #[test]
    fn with_files_to_process() {
        let (_, args) = parse_args(stringify(vec![
            basic_program_string(),
            "data1.txt",
            "data2.txt",
            "data3.txt",
        ]));
        assert_eq!(args.filepaths_to_parse.len(), 3);
        assert_eq!(
            args.filepaths_to_parse,
            stringify(vec!["data1.txt", "data2.txt", "data3.txt",])
        );
    }
}
