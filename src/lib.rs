extern crate nom;
extern crate regex;

use nom::{
    branch::alt,
    character::complete::multispace0,
    combinator::{all_consuming, map},
    multi::many1,
    sequence::delimited,
    IResult,
};
use std::collections::HashMap;

mod action;
mod basic_types;
mod expression;
mod function;
mod item;
mod parse_args;
mod pattern;
mod value;

use crate::{
    basic_types::{Context, Record},
    function::{parse_function, FunctionDefinition, Functions},
    item::{parse_item, Item},
};

pub struct Program {
    items: Vec<Item>,
    functions: Functions,
}

enum ParsedThing {
    Item(Item),
    Function(FunctionDefinition),
}

fn parse_item_list(input: &str) -> IResult<&str, (Vec<Item>, Vec<FunctionDefinition>)> {
    let parse_thing = alt((
        map(parse_item, |item| ParsedThing::Item(item)),
        map(parse_function, |function| ParsedThing::Function(function)),
    ));

    map(
        many1(delimited(multispace0, parse_thing, multispace0)),
        |things| {
            let mut items = vec![];
            let mut functions = vec![];

            for thing in things {
                match thing {
                    ParsedThing::Item(item) => {
                        items.push(item);
                    }
                    ParsedThing::Function(function) => {
                        functions.push(function);
                    }
                }
            }

            (items, functions)
        },
    )(input)
}

pub fn parse_program(program_text: &str) -> Program {
    match all_consuming(parse_item_list)(program_text) {
        Ok((_, (items, functions))) => {
            let mut function_map = HashMap::new();
            for func in functions {
                function_map.insert(func.name.clone(), func);
            }
            Program {
                items: items,
                functions: function_map,
            }
        }
        Err(e) => panic!("Could not parse! {}", e),
    }
}

pub struct ProgramRun {
    program: Program,
    context: Context,
}

pub fn start_run(args: Vec<String>) -> (ProgramRun, Vec<String>) {
    let (program_string, parsed_args) = parse_args::parse_args(args);
    let program = parse_program(&program_string);

    let mut run = ProgramRun {
        program: program,
        context: Context::empty(),
    };

    run.apply_args(&parsed_args);

    (run, parsed_args.filepaths_to_parse)
}

impl ProgramRun {
    pub fn output_for_line(&mut self, line: &str) -> Vec<String> {
        let record = Record {
            full_line: line,
            fields: self.split(line),
        };
        // Need explicit borrow of the context to avoid borrowing `self` later
        let context = &mut self.context;
        let functions = &self.program.functions;

        self.program
            .items
            .iter()
            .flat_map(|item: &item::Item| item.output_for_line(functions, context, &record))
            .collect()
    }

    pub fn output_for_begin_items(&mut self) -> Vec<String> {
        let context = &mut self.context;
        let functions = &self.program.functions;

        self.program
            .items
            .iter()
            .flat_map(|item| item.output_for_begin(functions, context))
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

    #[test]
    fn test_parse_program_with_function() {
        // Assert no panic
        let program = parse_program(
            r#"{ print(1);
            print(2.0);
            print("hello");
        }
        function foo(a) {
          print("hello");
        }
        "#,
        );
        assert_eq!(program.items.len(), 1);
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_bad_program() {
        // Assert no panic
        let program = parse_program(
            r#"function store(val) {
  a = val;
}
{
  print($0, a);
  b = store($0);
}"#,
        );
        assert_eq!(program.items.len(), 1);
        assert_eq!(program.functions.len(), 1);
    }
}
