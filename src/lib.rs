mod expression;
pub mod basic_types;
pub mod item;

use basic_types::Context;

pub struct Program {
    items: Vec<item::Item>,
}

pub fn parse_program(_program_text: &str) -> Program {
    Program {
        items: vec![
            item::Item {
                pattern: item::Pattern::MatchEverything,
                action: item::Action {
                    statements: vec![
                        item::Statement::Print(basic_types::Field::Indexed(3)),
                    ],
                }
            }
        ],
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
    pub fn output_for_line<'a>(&self, record: &basic_types::Record<'a>) -> Vec<&'a str> {
        self.program
            .items
            .iter()
            .filter(|item| {
                item.pattern.matches(record)
            })
            .flat_map(|item| {
                item.action.output_for_line(record)
            })
            .collect()
    }

    pub fn execute_begin(&mut self) {
        self.program.items.iter()
            .filter(|item| {
                match item.pattern {
                    item::Pattern::Begin => { true }
                    _ => { false }
                }
            })
            .for_each(|begin_rule| {
                self.execute_action(&begin_rule.action)
            });
    }

    fn execute_action(&mut self, action: &item::Action) {
    }
}
