extern crate nom;
extern crate regex;

use crate::basic_types::NumericValue;

use nom::{
    branch::alt,
    character::complete::{alpha1, multispace0, none_of, one_of},
    combinator::{map, map_res},
    multi::many1,
    re_find,
    sequence::{separated_pair, tuple},
    IResult,
};

use crate::{expression::Expression, item};

pub struct Program {
    pub items: Vec<item::Item>,
}

fn parse_expression(input: &str) -> IResult<&str, Expression> {
    alt((parse_addition, parse_literal, parse_variable))(input)
}

fn parse_variable(input: &str) -> IResult<&str, Expression> {
    map(alpha1, |name: &str| Expression::Variable(name.to_string()))(input)
}

fn parse_addition(input: &str) -> IResult<&str, Expression> {
    map(
        separated_pair(
            parse_expression,
            tuple((multispace0, one_of("+"), multispace0)),
            parse_expression,
        ),
        |(left, right)| Expression::AddBinary {
            left: Box::new(left),
            right: Box::new(right),
        },
    )(input)
}

fn parse_literal(input: &str) -> IResult<&str, Expression> {
    alt((
        parse_string_literal,
        parse_regex_literal,
        parse_number_literal,
    ))(input)
}

fn parse_float_literal(input: &str) -> IResult<&str, NumericValue> {
    let (input, matched) = re_find!(input, r"[-+]?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?")?;
    let number = matched.parse::<f64>().unwrap();

    IResult::Ok((input, NumericValue::Float(number)))
}

fn parse_integer_literal(input: &str) -> IResult<&str, NumericValue> {
    let (input, matched) = re_find!(input, r"[-+]?[0-9]+")?;
    let number = matched.parse::<i64>().unwrap();

    IResult::Ok((input, NumericValue::Integer(number)))
}

fn parse_number_literal(input: &str) -> IResult<&str, Expression> {
    map(
        alt((parse_integer_literal, parse_float_literal)),
        |number| Expression::NumericLiteral(number),
    )(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((one_of("\""), alpha1, one_of("\""))),
        |(_, contents, _): (char, &str, char)| Expression::StringLiteral(contents.to_string()),
    )(input)
}

fn parse_regex_literal(input: &str) -> IResult<&str, Expression> {
    let parser = tuple((one_of("/"), many1(none_of("/")), one_of("/")));
    map_res(parser, |(_, vec, _)| {
        regex::Regex::new(&vec.iter().collect::<String>()).map(|regex| Expression::Regex(regex))
    })(input)
}

pub fn parse_program(program_text: &str) -> Program {
    let default_program = Program {
        items: vec![item::Item {
            pattern: item::Pattern::MatchEverything,
            action: item::Action {
                statements: vec![item::Statement::Print(Expression::StringLiteral(
                    "hi".to_string(),
                ))],
            },
        }],
    };

    match parse_expression(program_text) {
        Ok((_, expr)) => Program {
            items: vec![item::Item {
                pattern: item::Pattern::MatchEverything,
                action: item::Action {
                    statements: vec![item::Statement::Print(expr)],
                },
            }],
        },
        _ => default_program,
    }
}
