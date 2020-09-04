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

// Regex matching does not associate
pub(super) fn parse_regex_match(input: &str) -> ExpressionParseResult {
    let (i, (left, operator, right)) = tuple((
        super::binary_math::parse_binary_math_expression,
        delimited(multispace0, alt((tag("~"), tag("!~"))), multispace0),
        super::binary_math::parse_binary_math_expression,
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
    fn test_regex_match() {
        let (context, record) = empty_context_and_record();

        let result = parse_regex_match("1 ~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0)),
        );

        // Cannot consume the full expression
        let result = parse_regex_match("1 ~ 2 ~ 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, " ~ 3");

        let result = parse_regex_match("1 + 2 ~ 3");
        assert!(result.is_ok());
        let (remainder, expression) = result.unwrap();
        assert_eq!(remainder, "");
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parse_regex_match("1 !~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );
    }
}
