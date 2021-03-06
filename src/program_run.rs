use std::io::{BufRead, BufReader, Read, Stdin};

use crate::{
    context::{MutableContext, VariableStore, Variables},
    parse_args,
    printable::Printable,
    program::Program,
    value::{NumericValue, Value},
};

pub struct ProgramRun {
    program: Program,
    variables: Variables,
}

type IOResult = std::io::Result<usize>;

pub trait LineReadable {
    fn trait_read_line(&mut self, buffer: &mut String) -> IOResult;
}

impl LineReadable for Stdin {
    fn trait_read_line(&mut self, buffer: &mut String) -> IOResult {
        self.read_line(buffer)
    }
}

impl<T: Read> LineReadable for BufReader<T> {
    fn trait_read_line(&mut self, buffer: &mut String) -> IOResult {
        self.read_line(buffer)
    }
}

impl ProgramRun {
    pub(crate) fn new_for_program(program: Program) -> ProgramRun {
        ProgramRun {
            program: program,
            variables: Variables::empty(),
        }
    }

    pub fn process_file<LR: LineReadable>(&mut self, reader: &mut LR) -> Vec<String> {
        self.variables
            .assign_variable("FNR", Value::Numeric(NumericValue::Integer(0)));

        let mut buffer = String::new();
        let mut output = vec![];
        loop {
            match reader.trait_read_line(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    if buffer.chars().last().unwrap() == '\n' {
                        buffer.truncate(n - 1);
                    }
                    output.append(&mut self.output_for_line(&buffer));
                    buffer.clear();
                }
                Err(error) => {
                    eprintln!("Error encountered: {}", error);
                    break;
                }
            }
        }
        output
    }

    fn output_for_line(&mut self, line: &str) -> Vec<String> {
        // Need explicit borrow of the variables to avoid borrowing `self` later
        let functions = &self.program.functions;
        self.variables.increment_variable("NR");
        self.variables.increment_variable("FNR");
        let mut context = MutableContext::for_variables(&mut self.variables);
        context.set_record_with_line(line);

        self.program
            .items
            .iter()
            .fold(Printable::wrap(()), |result, item| {
                result.and_then(|_| item.output_for_line(functions, &mut context))
            })
            .output
    }

    pub fn output_for_begin_items(&mut self) -> Vec<String> {
        let variables = &mut self.variables;
        let functions = &self.program.functions;
        let mut context = MutableContext::for_variables(variables);

        self.program
            .items
            .iter()
            .fold(Printable::wrap(()), |result, item| {
                result.and_then(|_| item.output_for_begin(functions, &mut context))
            })
            .output
    }

    pub(super) fn apply_args(&mut self, args: &parse_args::Args) {
        self.variables
            .assign_variable("FS", Value::String(args.field_separator.clone()));
        self.variables.assign_variable(
            "ARGC",
            Value::Numeric(NumericValue::Integer(args.filepaths_to_parse.len() as i64)),
        );

        for (name, value) in args.variables.iter() {
            self.variables
                .assign_variable(name, Value::String(value.to_string()));
        }
    }
}
