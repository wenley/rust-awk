use std::io::{BufRead, BufReader, Read, Stdin};

extern crate nom;
extern crate regex;

mod action;
mod basic_types;
mod expression;
mod function;
mod item;
mod parse_args;
mod pattern;
mod printable;
mod program;
mod value;

use crate::{
    basic_types::{MutableContext, VariableStore, Variables},
    printable::Printable,
};

pub struct ProgramRun {
    program: program::Program,
    variables: Variables,
}

pub fn start_run(args: Vec<String>) -> (ProgramRun, Vec<String>) {
    let (program_string, parsed_args) = parse_args::parse_args(args);
    let program = program::parse_program(&program_string);

    let mut run = ProgramRun {
        program: program,
        variables: Variables::empty(),
    };

    run.apply_args(&parsed_args);

    (run, parsed_args.filepaths_to_parse)
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
    pub fn process_file<LR: LineReadable>(&mut self, reader: &mut LR) -> Vec<String> {
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

    pub fn output_for_line(&mut self, line: &str) -> Vec<String> {
        // Need explicit borrow of the variables to avoid borrowing `self` later
        let functions = &self.program.functions;
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

        self.program
            .items
            .iter()
            .fold(Printable::wrap(()), |result, item| {
                result.and_then(|_| item.output_for_begin(functions, variables))
            })
            .output
    }

    pub fn apply_args(&mut self, args: &parse_args::Args) {
        self.variables
            .assign_variable("FS", value::Value::String(args.field_separator.clone()));
        for (name, value) in args.variables.iter() {
            self.variables
                .assign_variable(name, value::Value::String(value.to_string()));
        }
    }
}
