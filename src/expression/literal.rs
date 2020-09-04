use nom::{
    branch::alt,
    character::complete::{none_of, one_of},
    multi::many1,
    re_find,
    sequence::{delimited, tuple},
    IResult,
};

use crate::{
    basic_types::{Context, Record},
    value::{parse_float_literal, parse_integer_literal, NumericValue, Value},
};

use super::{Expression, ExpressionResult};
use regex::Regex;

#[derive(Debug)]
pub(super) enum Literal {
    String(String),
    Numeric(NumericValue),
    Regex(Regex),
}

impl Expression for Literal {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        match self {
            Literal::Regex(r) => Some(r),
            _ => None,
        }
    }

    fn evaluate<'a>(&self, _context: &Context, _record: &'a Record) -> Value {
        match self {
            Literal::String(string) => Value::String(string.clone()),
            Literal::Numeric(numeric) => Value::Numeric(numeric.clone()),
            Literal::Regex(_) => {
                // Regex expressions shouldn't be evaluated as a standalone value, but should be
                // evaluated as part of explicit pattern matching operators. The one exception is
                // when a Regex is the pattern for an action, which is handled separately.
                //
                // When a Regex is evaluated on its own, it becomes an empty numeric string and
                // will be interpreted as such
                Value::Uninitialized
            }
        }
    }
}

pub(super) fn parse_literal(input: &str) -> ExpressionResult {
    alt((
        parse_string_literal,
        parse_regex_literal,
        parse_number_literal,
    ))(input)
}

fn parse_number_literal(input: &str) -> ExpressionResult {
    let (i, number) = alt((parse_float_literal, parse_integer_literal))(input)?;

    Result::Ok((i, Box::new(Literal::Numeric(number))))
}

fn parse_string_literal(input: &str) -> ExpressionResult {
    let (i, contents) = delimited(one_of("\""), parse_string_contents, one_of("\""))(input)?;

    Result::Ok((i, Box::new(Literal::String(contents.to_string()))))
}

fn parse_string_contents(input: &str) -> IResult<&str, &str> {
    //
    // Allow strings to contain sequences of:
    // 1. Non-slash non-double-quote characters
    // 2. Slash followed anything (including a double-quote)
    //
    re_find!(input, r#"^([^\\"]|\\.)*"#)
}

use nom::error::ErrorKind;
use nom::error::ParseError;
use nom::Err;
fn parse_regex_literal(input: &str) -> ExpressionResult {
    let (i, (_, vec, _)) = tuple((one_of("/"), many1(none_of("/")), one_of("/")))(input)?;

    let result = regex::Regex::new(&vec.iter().collect::<String>());
    match result {
        Ok(r) => Result::Ok((i, Box::new(Literal::Regex(r)))),
        Err(_) => Result::Err(Err::Error(ParseError::from_error_kind(
            i,
            ErrorKind::MapRes,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_context_and_record() -> (Context, Record<'static>) {
        (
            Context::empty(),
            Record {
                full_line: "",
                fields: vec![],
            },
        )
    }

    #[test]
    fn literals_can_evaluate() {
        let (context, record) = empty_context_and_record();
        let string = Literal::String("hello".to_string());
        assert_eq!(
            string.evaluate(&context, &record),
            Value::String("hello".to_string())
        );
        let numeric = Literal::Numeric(NumericValue::Integer(0));
        assert_eq!(
            numeric.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0))
        );
    }

    #[test]
    fn test_parse_literals() {
        let (context, record) = empty_context_and_record();

        let result = parse_literal("1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1))
        );

        let result = parse_literal(r#""hello""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::String("hello".to_string()),
        );

        let result = parse_literal(r#""hello world""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::String("hello world".to_string()),
        );
    }
}
