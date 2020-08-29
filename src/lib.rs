extern crate nom;
extern crate regex;

pub mod basic_types;
mod expression;
pub mod item;
mod pattern;
mod statement;
mod value;

use crate::{
    basic_types::Context,
    expression::Expression,
    item::{parse_item_list, Item},
    pattern::Pattern,
    statement::{Action, Statement},
};

pub struct Program {
    pub items: Vec<Item>,
}

pub fn parse_program(program_text: &str) -> Program {
    let default_program = Program {
        items: vec![Item {
            pattern: Pattern::MatchEverything,
            action: Action {
                statements: vec![Statement::Print(vec![Expression::StringLiteral(
                    "hi".to_string(),
                )])],
            },
        }],
    };

    match parse_item_list(program_text) {
        Ok((_, items)) => Program { items: items },
        _ => default_program,
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
    pub fn output_for_line<'a>(&mut self, record: &basic_types::Record<'a>) -> Vec<String> {
        self.program
            .items
            .iter()
            .filter(|item| item.pattern.matches(record))
            .flat_map(|item| item.action.output_for_line(&mut self.context, record))
            .collect()
    }

    pub fn execute_begin(&mut self) {
        self.program
            .items
            .iter()
            .filter(|item| match item.pattern {
                Pattern::Begin => true,
                _ => false,
            })
            .for_each(|begin_rule| self.execute_action(&begin_rule.action));
    }

    fn execute_action(&mut self, action: &Action) {}

    pub fn split<'a>(&self, line: &'a str) -> Vec<&'a str> {
        self.context.split(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::NumericValue;

    #[test]
    fn test_parse_program() {
        let program = parse_program(
            r#"{ print(1);
            print(2.0);
            print("hello");
        }"#,
        );

        assert_eq!(program.items[0].action.statements.len(), 3);
    }
}
