use regex::Regex;
use std::fmt::Debug;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    sequence::{delimited, tuple},
};

use super::{Expression, ExpressionParseResult};
use crate::{
    basic_types::{Context, Record},
    value::{NumericValue, Value},
};

#[derive(Debug)]
struct RegexMatch {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
    negated: bool,
}

impl Expression for RegexMatch {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        let left_value = self.left.evaluate(context, record).coerce_to_string();

        let matches = match self.right.regex() {
            Some(r) => r.is_match(&left_value),
            None => {
                let right_value = self.right.evaluate(context, record).coerce_to_string();
                Regex::new(&right_value).unwrap().is_match(&left_value)
            }
        };
        let int_value = if matches ^ self.negated { 1 } else { 0 };

        Value::Numeric(NumericValue::Integer(int_value))
    }
}

pub(super) fn regex_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        let (i, (left, operator, right)) = tuple((
            |i| next_parser(i),
            delimited(multispace0, alt((tag("~"), tag("!~"))), multispace0),
            |i| next_parser(i),
        ))(input)?;

        let negated = match operator {
            "~" => false,
            "!~" => true,
            _ => panic!("Unexpected regex operator length: {}", operator),
        };
        Result::Ok((
            i,
            Box::new(RegexMatch {
                left: left,
                right: right,
                negated,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::binary_math::parse_binary_math_expression;
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
    fn test_regex_match() {
        let (context, record) = empty_context_and_record();

        let result = regex_parser(parse_binary_math_expression)("1 ~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0)),
        );

        // Cannot consume the full expression
        let result = regex_parser(parse_binary_math_expression)("1 ~ 2 ~ 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, " ~ 3");

        let result = regex_parser(parse_binary_math_expression)("1 + 2 ~ 3");
        assert!(result.is_ok());
        let (remainder, expression) = result.unwrap();
        assert_eq!(remainder, "");
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = regex_parser(parse_binary_math_expression)("1 !~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );
    }
}
