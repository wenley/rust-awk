use regex::Regex;

use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::map,
    multi::many0,
    sequence::{delimited, pair, preceded},
};

use super::{Expression, ExpressionParseResult};
use crate::{
    basic_types::{Context, Record},
    value::{NumericValue, Value},
};

#[derive(Debug)]
enum Operator {
    And,
}

#[derive(Debug)]
struct BinaryBoolean {
    left: Box<dyn Expression>,
    operator: Operator,
    right: Box<dyn Expression>,
}

impl Expression for BinaryBoolean {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        let left_value = self.left.evaluate(context, record).coercion_to_boolean();
        let right_value = self.right.evaluate(context, record).coercion_to_boolean();

        let result = match &self.operator {
            Operator::And => left_value && right_value,
        };
        let int_value = if result { 1 } else { 0 };
        Value::Numeric(NumericValue::Integer(int_value))
    }
}

pub(super) fn and_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        let parse_added_expr = preceded(delimited(multispace0, tag("&&"), multispace0), |i| {
            next_parser(i)
        });
        map(
            pair(|i| next_parser(i), many0(parse_added_expr)),
            move |(first, mut rest)| {
                rest.drain(0..).fold(first, |inner, next| {
                    Box::new(BinaryBoolean {
                        left: inner,
                        operator: Operator::And,
                        right: next,
                    })
                })
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::super::literal::*;
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
    fn test_and_parsing() {
        let (context, record) = empty_context_and_record();
        let parser = and_parser(parse_literal);

        let result = parser(r#""a" && 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser(r#""a" && 0"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0)),
        );

        let result = parser(r#""" && 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0)),
        );
    }
}
