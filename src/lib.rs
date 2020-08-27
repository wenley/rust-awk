pub mod basic_types;
mod expression;
pub mod item;
pub mod parse;
mod pattern;
mod statement;
mod value;

use basic_types::Context;
use item::Action;
use pattern::Pattern;

pub struct ProgramRun<'a> {
    program: &'a parse::Program,
    context: Context,
}

pub fn start_run<'a>(program: &'a parse::Program) -> ProgramRun<'a> {
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
