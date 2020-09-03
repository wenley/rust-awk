use regex::Regex;
use std::fmt::Debug;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, one_of},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::{
    basic_types::{Context, Record},
    value::{NumericValue, Value},
};

mod binary_math;
mod literal;
mod variable;

pub(crate) trait Expression: Debug {
    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value;

    fn regex<'a>(&'a self) -> Option<&'a Regex>;
}

#[derive(Debug)]
enum ExpressionImpl {
    FieldReference(Box<dyn Expression>),
    RegexMatch {
        left: Box<dyn Expression>,
        right: Box<dyn Expression>,
        negated: bool,
    },
}

impl Expression for ExpressionImpl {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        match self {
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

                let matches = match right.regex() {
                    Some(r) => r.is_match(&left_value),
                    None => {
                        let right_value = right.evaluate(context, record).coerce_to_string();
                        Regex::new(&right_value).unwrap().is_match(&left_value)
                    }
                };
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
    alt((parse_regex_match, binary_math::parse_addition))(input)
}

// Regex matching does not associate
fn parse_regex_match(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, (left, operator, right)) = tuple((
        binary_math::parse_addition,
        delimited(multispace0, alt((tag("~"), tag("!~"))), multispace0),
        binary_math::parse_addition,
    ))(input)?;

    let negated = match operator.len() {
        1 => false,
        2 => true,
        _ => panic!("Unexpected regex operator length: {}", operator),
    };
    Result::Ok((
        i,
        Box::new(ExpressionImpl::RegexMatch {
            left: left,
            right: right,
            negated,
        }),
    ))
}

fn parse_primary(input: &str) -> IResult<&str, Box<dyn Expression>> {
    alt((
        literal::parse_literal,
        variable::parse_variable,
        parse_parens,
        parse_field_reference,
    ))(input)
}

fn parse_parens(input: &str) -> IResult<&str, Box<dyn Expression>> {
    delimited(one_of("("), parse_expression, one_of(")"))(input)
}

fn parse_field_reference(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, expr) = preceded(one_of("$"), parse_expression)(input)?;
    Result::Ok((i, Box::new(ExpressionImpl::FieldReference(expr))))
}

#[cfg(test)]
mod tests {
    use super::literal::*;
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
    fn field_reference_can_evaluate() {
        let (context, mut record) = empty_context_and_record();
        record.fields = vec!["first", "second"];

        assert_eq!(
            ExpressionImpl::FieldReference(Box::new(Literal::Numeric(NumericValue::Integer(1))))
                .evaluate(&context, &record),
            Value::String("first".to_string()),
        );
    }

    #[test]
    fn test_parse_parens() {
        let (context, record) = empty_context_and_record();

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
