extern crate nom;
extern crate regex;

use crate::basic_types::NumericValue;

use nom::{
    branch::alt,
    character::complete::{alpha1, multispace0, none_of, one_of},
    combinator::{map, map_res},
    multi::{many0, many1},
    re_find,
    sequence::{delimited, pair, tuple},
    IResult,
};

use crate::{expression::Expression, item};

pub struct Program {
    pub items: Vec<item::Item>,
}

/* - - - - - - - - - -
 * Expression Parsers
 * in increasing order of precedence
 * - - - - - - - - - - */

fn parse_expression(input: &str) -> IResult<&str, Expression> {
    parse_addition(input)
}

fn parse_addition(input: &str) -> IResult<&str, Expression> {
    let parse_added_expr = map(
        pair(delimited(multispace0, one_of("+"), multispace0), parse_primary),
        |(_, rhs)| rhs
    );
    map(
        pair(parse_primary, many0(parse_added_expr)),
        move |(first, mut rest)| rest.drain(0..).fold(first, |inner, next| {
            Expression::AddBinary {
                left: Box::new(inner),
                right: Box::new(next),
            }
        })
    )(input)
}

fn parse_primary(input: &str) -> IResult<&str, Expression> {
    alt((parse_literal, parse_variable, parse_parens))(input)
}

fn parse_parens(input: &str) -> IResult<&str, Expression> {
    delimited(one_of("("), parse_expression, one_of(")"))(input)
}

fn parse_variable(input: &str) -> IResult<&str, Expression> {
    map(alpha1, |name: &str| Expression::Variable(name.to_string()))(input)
}

fn parse_literal(input: &str) -> IResult<&str, Expression> {
    alt((
        parse_string_literal,
        parse_regex_literal,
        parse_number_literal,
    ))(input)
}

fn parse_float_literal(input: &str) -> IResult<&str, NumericValue> {
    // Omit ? on the . to intentionally _not_ match on integers
    let (input, matched) = re_find!(input, r"^[-+]?[0-9]*\.[0-9]+([eE][-+]?[0-9]+)?")?;
    let number = matched.parse::<f64>().unwrap();

    IResult::Ok((input, NumericValue::Float(number)))
}

fn parse_integer_literal(input: &str) -> IResult<&str, NumericValue> {
    let (input, matched) = re_find!(input, r"^[-+]?[0-9]+")?;
    let number = matched.parse::<i64>().unwrap();

    IResult::Ok((input, NumericValue::Integer(number)))
}

fn parse_number_literal(input: &str) -> IResult<&str, Expression> {
    map(
        alt((parse_float_literal, parse_integer_literal)),
        |number| Expression::NumericLiteral(number),
    )(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Expression> {
    map(
        delimited(one_of("\""), alpha1, one_of("\"")),
        |contents: &str| Expression::StringLiteral(contents.to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number_literals() {
        // Integers
        assert_eq!(
            parse_integer_literal("123").unwrap().1,
            NumericValue::Integer(123)
        );
        assert_eq!(
            parse_integer_literal("123000").unwrap().1,
            NumericValue::Integer(123000)
        );
        assert_eq!(
            parse_integer_literal("-123").unwrap().1,
            NumericValue::Integer(-123)
        );
        assert_eq!(parse_integer_literal("(123").is_err(), true);
        // Would like this test to pass, but the distinction is implemented
        // by the sequencing of the parsers of parse_number_literal
        // assert_eq!(parse_integer_literal("123.45").is_err(), true);
        assert_eq!(parse_integer_literal(".").is_err(), true);

        // Floats
        assert_eq!(
            parse_float_literal("123.45"),
            IResult::Ok(("", NumericValue::Float(123.45)))
        );
        assert_eq!(
            parse_float_literal("123.45e-5"),
            IResult::Ok(("", NumericValue::Float(123.45e-5)))
        );
        assert_eq!(
            parse_float_literal("123.45E5"),
            IResult::Ok(("", NumericValue::Float(123.45e5)))
        );
        assert_eq!(
            parse_float_literal(".45"),
            IResult::Ok(("", NumericValue::Float(0.45)))
        );
        assert_eq!(
            parse_float_literal("-123.45"),
            IResult::Ok(("", NumericValue::Float(-123.45)))
        );
        assert_eq!(parse_float_literal("a").is_err(), true);
        assert_eq!(parse_float_literal(".").is_err(), true);
        assert_eq!(parse_float_literal("+e").is_err(), true);

        // Cannot test parse_number_literal because macros don't reveal types
    }

    #[test]
    fn parse_expressions() {
        let result = parse_expression("1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, Expression::NumericLiteral(NumericValue::Integer(1)));

        let result = parse_expression("(1)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, Expression::NumericLiteral(NumericValue::Integer(1)));

        let result = parse_expression("(1) + (2.5)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, Expression::AddBinary {
            left: Box::new(Expression::NumericLiteral(NumericValue::Integer(1))),
            right: Box::new(Expression::NumericLiteral(NumericValue::Float(2.5))),
        });
        let result = parse_expression("(1) + (2.5)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, Expression::AddBinary {
            left: Box::new(Expression::NumericLiteral(NumericValue::Integer(1))),
            right: Box::new(Expression::NumericLiteral(NumericValue::Float(2.5))),
        });
    }
}
