use crate::{
    expression::Expression,
    item
};

pub struct Program {
    pub items: Vec<item::Item>,
}

pub fn parse_program(_program_text: &str) -> Program {
    Program {
        items: vec![item::Item {
            pattern: item::Pattern::MatchEverything,
            action: item::Action {
                statements: vec![item::Statement::Print(
                    Expression::StringLiteral("hi".to_string()),
                )],
            },
        }],
    }
}

