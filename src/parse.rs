extern crate nom;

use nom::{
    IResult,
    character::complete::{
        one_of,
        alpha1,
    },
};

use crate::{
    expression::Expression,
    item
};

pub struct Program {
    pub items: Vec<item::Item>,
}

fn parse_string_literal(input: &str) -> IResult<&str, Expression> {
    let (input, _) = one_of("\"")(input)?;
    let (input, contents) = alpha1(input)?;
    let (input, _) = one_of("\"")(input)?;

    IResult::Ok((input, Expression::StringLiteral(contents.to_string())))
}

pub fn parse_program(program_text: &str) -> Program {
    let default_program = Program {
        items: vec![item::Item {
            pattern: item::Pattern::MatchEverything,
            action: item::Action {
                statements: vec![item::Statement::Print(
                                Expression::StringLiteral("hi".to_string()),
                                )],
            },
        }],
    };

    match parse_string_literal(program_text) {
        Ok((_, expr)) => {
            Program {
                items: vec![item::Item {
                    pattern: item::Pattern::MatchEverything,
                    action: item::Action {
                        statements: vec![item::Statement::Print(expr)],
                    },
                }]
            }
        }
        _ => { default_program }
    }
}

