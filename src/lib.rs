extern crate nom;
extern crate regex;

mod action;
mod basic_types;
mod expression;
mod item;
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

pub struct ProgramRun<'a> {
    program: &'a Program,
    context: Context,
}

pub fn start_run<'a>(program: &'a Program) -> ProgramRun<'a> {
    ProgramRun {
        program: program,
        context: Context::empty(),
    }
}

impl ProgramRun<'_> {
    pub fn output_for_line(&mut self, line: &str) -> Vec<String> {
        let record = Record {
            full_line: line,
            fields: self.split(line),
        };

        self.program
            .items
            .iter()
            .flat_map(|item| item.output_for_line(&mut self.context, &record))
            .collect()
    }

    pub fn output_for_begin_items(&mut self) -> Vec<String> {
        self.program
            .items
            .iter()
            .flat_map(|item| item.output_for_begin(&mut self.context))
            .collect()
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
