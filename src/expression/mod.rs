use regex::Regex;
use std::fmt::Debug;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, multispace0, none_of, one_of},
    combinator::{map},
    multi::{many0, many1},
    re_find,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

use crate::{
    basic_types::{Context, Record},
    value::{parse_float_literal, parse_integer_literal, NumericValue, Value},
};

pub(crate) trait Expression: Debug {
    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value;
}

#[derive(Debug)]
enum ExpressionImpl {
    StringLiteral(String),
    NumericLiteral(NumericValue),
    AddBinary {
        left: Box<dyn Expression>,
        right: Box<dyn Expression>,
    },
    Regex(Regex),
    Variable(String),
    FieldReference(Box<dyn Expression>),
    RegexMatch {
        left: Box<dyn Expression>,
        right: Box<dyn Expression>,
        negated: bool,
    },
}

impl Expression for ExpressionImpl {
    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        match self {
            ExpressionImpl::StringLiteral(string) => Value::String(string.clone()),
            ExpressionImpl::NumericLiteral(numeric) => Value::Numeric(numeric.clone()),
            ExpressionImpl::AddBinary { left, right } => {
                match (
                    left.evaluate(context, record).coerce_to_numeric(),
                    right.evaluate(context, record).coerce_to_numeric(),
                ) {
                    (NumericValue::Integer(x), NumericValue::Integer(y)) => {
                        Value::Numeric(NumericValue::Integer(x + y))
                    }
                    (NumericValue::Integer(x), NumericValue::Float(y)) => {
                        Value::Numeric(NumericValue::Float((x as f64) + y))
                    }
                    (NumericValue::Float(x), NumericValue::Integer(y)) => {
                        Value::Numeric(NumericValue::Float(x + (y as f64)))
                    }
                    (NumericValue::Float(x), NumericValue::Float(y)) => {
                        Value::Numeric(NumericValue::Float(x + y))
                    }
                }
            }
            ExpressionImpl::Regex(_) => {
                // Regex expressions shouldn't be evaluated as a standalone value, but should be
                // evaluated as part of explicit pattern matching operators. The one exception is
                // when a Regex is the pattern for an action, which is handled separately.
                //
                // When a Regex is evaluated on its own, it becomes an empty numeric string and
                // will be interpreted as such
                Value::Uninitialized
            }
            ExpressionImpl::Variable(variable_name) => context.fetch_variable(variable_name),
            ExpressionImpl::FieldReference(expression) => {
                let value = expression.evaluate(context, record).coerce_to_numeric();
                let unsafe_index = match value {
                    NumericValue::Integer(i) => i,
                    NumericValue::Float(f) => f.floor() as i64,
                };
                match unsafe_index {
                    i if i < 0 => panic!("Field indexes cannot be negative: {}", unsafe_index),
                    i if i == 0 => Value::String(record.full_line.to_string()),
                    i => record
                        .fields
                        .get((i - 1) as usize)
                        .map(|s| Value::String(s.to_string()))
                        .unwrap_or(Value::Uninitialized),
                }
            }
            ExpressionImpl::RegexMatch {
                left,
                right,
                negated,
            } => {
                let left_value = left.evaluate(context, record).coerce_to_string();

                let right_value = right.evaluate(context, record).coerce_to_string();
                let matches = Regex::new(&right_value).unwrap().is_match(&left_value);
                // let matches = match &**right {
                //     // ExpressionImpl::Regex(regex) => regex.is_match(&left_value),
                //     _ => {
                //         let value = right.evaluate(context, record).coerce_to_string();
                //         Regex::new(&value).unwrap().is_match(&left_value)
                //     }
                // };

                let int_value = if matches ^ negated { 1 } else { 0 };

                Value::Numeric(NumericValue::Integer(int_value))
            }
        }
    }
}

/// Tiers of parsing
///
/// The top-level parser is responsible for the loosest-binding / lowest-precedence
/// operators. As we descend the levels, we encounter tighter-binding operators
/// until we reach literals and the parenthesized expressions.

pub(crate) fn parse_expression(input: &str) -> IResult<&str, Box<dyn Expression>> {
    alt((parse_regex_match, parse_addition))(input)
}

// Regex matching does not associate
fn parse_regex_match(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, (left, operator, right)) = tuple((
        parse_addition,
        delimited(multispace0, alt((tag("~"), tag("!~"))), multispace0),
        parse_addition,
    ))(input)?;

    let negated = match operator.len() {
        1 => false,
        2 => true,
        _ => panic!("Unexpected regex operator length: {}", operator),
    };
    Result::Ok((i, Box::new(ExpressionImpl::RegexMatch {
        left: left,
        right: right,
        negated,
    })))
}

fn parse_addition(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let parse_added_expr = map(
        pair(
            delimited(multispace0, one_of("+"), multispace0),
            parse_primary,
        ),
        |(_, rhs)| rhs,
    );
    // Why does this `map` work??
    map(
        pair(parse_primary, many0(parse_added_expr)),
        move |(first, mut rest)| {
            rest.drain(0..)
                .fold(first, |inner, next| Box::new(ExpressionImpl::AddBinary {
                    left: inner,
                    right: next,
                }))
        },
    )(input)
}

fn parse_primary(input: &str) -> IResult<&str, Box<dyn Expression>> {
    alt((
        parse_literal,
        parse_variable,
        parse_parens,
        parse_field_reference,
    ))(input)
}

fn parse_parens(input: &str) -> IResult<&str, Box<dyn Expression>> {
    delimited(one_of("("), parse_expression, one_of(")"))(input)
}

fn parse_variable(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, name) = alpha1(input)?;

    Result::Ok((i, Box::new(ExpressionImpl::Variable(name.to_string()))))
}

fn parse_literal(input: &str) -> IResult<&str, Box<dyn Expression>> {
    alt((
        parse_string_literal,
        parse_regex_literal,
        parse_number_literal,
    ))(input)
}

fn parse_number_literal(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, number) = alt((parse_float_literal, parse_integer_literal))(input)?;

    Result::Ok((i, Box::new(ExpressionImpl::NumericLiteral(number))))
}

fn parse_string_literal(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, contents) = delimited(one_of("\""), parse_string_contents, one_of("\""))(input)?;

    Result::Ok((i, Box::new(ExpressionImpl::StringLiteral(contents.to_string()))))
}

fn parse_string_contents(input: &str) -> IResult<&str, &str> {
    //
    // Allow strings to contain sequences of:
    // 1. Non-slash non-double-quote characters
    // 2. Slash followed anything (including a double-quote)
    //
    re_find!(input, r#"^([^\\"]|\\.)*"#)
}

use nom::error::ParseError;
use nom::error::ErrorKind;
use nom::Err;
fn parse_regex_literal(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, (_, vec, _)) = tuple((one_of("/"), many1(none_of("/")), one_of("/")))(input)?;

    let result = regex::Regex::new(&vec.iter().collect::<String>());
    match result {
        Ok(r) => Result::Ok((i, Box::new(ExpressionImpl::Regex(r)))),
        Err(_) => Result::Err(Err::Error(ParseError::from_error_kind(i, ErrorKind::MapRes)))
    }
}

fn parse_field_reference(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, expr) = preceded(one_of("$"), parse_expression)(input)?;
    Result::Ok((i, Box::new(ExpressionImpl::FieldReference(expr))))
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
        let string = ExpressionImpl::StringLiteral("hello".to_string());
        assert_eq!(
            string.evaluate(&context, &record),
            Value::String("hello".to_string())
        );
        let numeric = ExpressionImpl::NumericLiteral(NumericValue::Integer(0));
        assert_eq!(
            numeric.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0))
        );
    }

    #[test]
    fn variables_can_evaluate() {
        let (mut context, record) = empty_context_and_record();
        let value = Value::Numeric(NumericValue::Integer(1));
        context.assign_variable("foo", value.clone());

        assert_eq!(
            ExpressionImpl::Variable("foo".to_string()).evaluate(&context, &record),
            value,
        );
    }

    #[test]
    fn binary_expressions_can_evaluate() {
        let (context, record) = empty_context_and_record();
        assert_eq!(
            ExpressionImpl::AddBinary {
                left: Box::new(ExpressionImpl::NumericLiteral(NumericValue::Integer(2))),
                right: Box::new(ExpressionImpl::NumericLiteral(NumericValue::Integer(3))),
            }
            .evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(5)),
        );
    }

    #[test]
    fn field_reference_can_evaluate() {
        let (context, mut record) = empty_context_and_record();
        record.fields = vec!["first", "second"];

        assert_eq!(
            ExpressionImpl::FieldReference(Box::new(ExpressionImpl::NumericLiteral(
                NumericValue::Integer(1)
            )))
            .evaluate(&context, &record),
            Value::String("first".to_string()),
        );
    }

    #[test]
    fn parse_integer_literal() {
        let (context, record) = empty_context_and_record();

        let result = parse_expression("1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1))
        );

        let result = parse_string_literal(r#""hello""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::String("hello".to_string()),
        );

        let result = parse_expression(r#""hello world""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::String("hello world".to_string()),
        );

        let result = parse_expression("(1)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1))
        );

        let result = parse_expression("(1) + (2.5)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Float(3.5))
        );
    }

    #[test]
    fn test_parse_field_reference() {
        let (context, mut record) = empty_context_and_record();
        let result = parse_expression("$1");
        assert_eq!(result.is_ok(), true);
        let expression = result.unwrap().1;
        assert_eq!(expression.evaluate(&context, &record), Value::Uninitialized,);

        record.fields = vec!["hello"];
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::String("hello".to_string()),
        );
    }

    #[test]
    fn test_regex_match() {
        let (context, record) = empty_context_and_record();

        let result = parse_expression("1 ~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0)),
        );

        // Cannot consume the full expression
        let result = parse_expression("1 ~ 2 ~ 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, " ~ 3");

        let result = parse_expression("1 + 2 ~ 3");
        assert!(result.is_ok());
        let (remainder, expression) = result.unwrap();
        assert_eq!(remainder, "");
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parse_expression("1 !~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );
    }
}
