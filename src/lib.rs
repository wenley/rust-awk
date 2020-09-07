extern crate nom;
extern crate regex;

mod action;
mod basic_types;
mod expression;
mod item;
mod parse_args;
mod pattern;
mod value;

use crate::{
    basic_types::{Context, Record},
    item::{parse_item_list, Item},
};

pub struct Program {
    items: Vec<Item>,
}

pub fn parse_program(program_text: &str) -> Program {
    match parse_item_list(program_text) {
        Ok((_, items)) => Program { items: items },
        Err(e) => panic!("Could not parse! {}", e),
    }
}

pub struct ProgramRun {
    program: Program,
    context: Context,
}

pub fn start_run(args: Vec<String>) -> ProgramRun {
    let parsed_args = parse_args::parse_args(args);
    let program = parse_program(&parsed_args.program_string.clone().unwrap());

    let mut run = ProgramRun {
        program: program,
        context: Context::empty(),
    };

    run.apply_args(&parsed_args);

    run
}

impl ProgramRun {
    pub fn output_for_line(&mut self, line: &str) -> Vec<String> {
        let record = Record {
            full_line: line,
            fields: self.split(line),
        };
        // Need explicit borrow of the context to avoid borrowing `self` later
        let context = &mut self.context;

        self.program
            .items
            .iter()
            .flat_map(|item: &item::Item| item.output_for_line(context, &record))
            .collect()
    }

    pub fn output_for_begin_items(&mut self) -> Vec<String> {
        let context = &mut self.context;
        self.program
            .items
            .iter()
            .flat_map(|item| item.output_for_begin(context))
            .collect()
    }

    pub fn apply_args(&mut self, args: &parse_args::Args) {
        self.context.set_field_separator(&args.field_separator);
        for (name, value) in args.variables.iter() {
            self.context
                .assign_variable(name, value::Value::String(value.to_string()));
        }
    }

    fn split<'a>(&self, line: &'a str) -> Vec<&'a str> {
        self.context.split(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program() {
        // Assert no panic
        parse_program(
            r#"{ print(1);
            print(2.0);
            print("hello");
        }"#,
        );
    }
}
