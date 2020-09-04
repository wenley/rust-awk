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
    value::{NumericValue, Value},
};

use super::{Expression, ExpressionParseResult};
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

pub(super) fn parse_literal(input: &str) -> ExpressionParseResult {
    alt((
        parse_string_literal,
        parse_regex_literal,
        parse_number_literal,
    ))(input)
}

fn parse_number_literal(input: &str) -> ExpressionParseResult {
    let (i, number) = alt((parse_float_literal, parse_integer_literal))(input)?;

    Result::Ok((i, Box::new(Literal::Numeric(number))))
}

pub(crate) fn parse_float_literal(input: &str) -> IResult<&str, NumericValue> {
    // Omit ? on the . to intentionally _not_ match on integers
    let (input, matched) = re_find!(input, r"^[-+]?[0-9]*\.[0-9]+([eE][-+]?[0-9]+)?")?;
    let number = matched.parse::<f64>().unwrap();

    IResult::Ok((input, NumericValue::Float(number)))
}

pub(crate) fn parse_integer_literal(input: &str) -> IResult<&str, NumericValue> {
    let (input, matched) = re_find!(input, r"^[-+]?[0-9]+")?;
    let number = matched.parse::<i64>().unwrap();

    IResult::Ok((input, NumericValue::Integer(number)))
}

fn parse_string_literal(input: &str) -> ExpressionParseResult {
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
fn parse_regex_literal(input: &str) -> ExpressionParseResult {
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
    }
}
