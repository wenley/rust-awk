extern crate nom;
extern crate regex;

use nom::{
    IResult,
    branch::alt,
    character::complete::{
        one_of,
        alpha1,
        none_of,
    },
    combinator::{
        map_res,
        not,
    },
    multi::{
        many1,
    }
};

use crate::{
    expression::Expression,
    item
};

pub struct Program {
    pub items: Vec<item::Item>,
}

fn parse_literal(input: &str) -> IResult<&str, Expression> {
    alt((parse_string_literal, parse_regex_literal))(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Expression> {
    let (input, _) = one_of("\"")(input)?;
    let (input, contents) = alpha1(input)?;
    let (input, _) = one_of("\"")(input)?;

    IResult::Ok((input, Expression::StringLiteral(contents.to_string())))
}

fn parse_regex_literal(input: &str) -> IResult<&str, Expression> {
    let (input, _) = one_of("/")(input)?;
    let (input, contents) = map_res(
        many1(none_of("/")),
        |vec| regex::Regex::new(&vec.iter().collect::<String>())
    )(input)?;
    let (input, _) = one_of("/")(input)?;

    IResult::Ok((input, Expression::Regex(contents)))
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

    match parse_literal(program_text) {
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

