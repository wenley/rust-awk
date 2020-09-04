use regex::Regex;

use nom::{
    branch::alt,
    character::complete::{multispace0, one_of},
    combinator::map,
    sequence::{delimited, tuple},
};

use super::{Expression, ExpressionParseResult};
use crate::{
    basic_types::{Context, Record},
    value::{NumericValue, Value},
};

#[derive(Debug)]
enum Operator {
    Less,
}

#[derive(Debug)]
struct BinaryComparison {
    left: Box<dyn Expression>,
    operator: Operator,
    right: Box<dyn Expression>,
}

impl Expression for BinaryComparison {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        let left_value = self.left.evaluate(context, record);
        let right_value = self.right.evaluate(context, record);

        let result = if let (Value::Numeric(left_number), Value::Numeric(right_number)) =
            (&left_value, &right_value)
        {
            let (x, y) = match (left_number, right_number) {
                (NumericValue::Integer(x), NumericValue::Integer(y)) => ((*x as f64), (*y as f64)),
                (NumericValue::Integer(x), NumericValue::Float(y)) => ((*x as f64), *y),
                (NumericValue::Float(x), NumericValue::Integer(y)) => (*x, (*y as f64)),
                (NumericValue::Float(x), NumericValue::Float(y)) => (*x, *y),
            };
            match self.operator {
                Operator::Less => x < y,
            }
        } else {
            let (s1, s2) = (
                left_value.coerce_to_string(),
                right_value.coerce_to_string(),
            );
            match &self.operator {
                Operator::Less => s1 < s2,
            }
        };

        let int_value = if result { 1 } else { 0 };
        Value::Numeric(NumericValue::Integer(int_value))
    }
}

pub(super) fn parse_binary_comparison(input: &str) -> ExpressionParseResult {
    alt((parse_definite_comparison, super::literal::parse_literal))(input)
}

fn parse_definite_comparison(input: &str) -> ExpressionParseResult {
    let parse_operator = map(
        delimited(multispace0, one_of("<"), multispace0),
        |operator| match operator {
            '<' => Operator::Less,
            _ => panic!("Unrecognized comparison character: {}", operator),
        },
    );

    let (i, (left, operator, right)) = tuple((
        super::literal::parse_literal,
        parse_operator,
        super::literal::parse_literal,
    ))(input)?;

    Result::Ok((
        i,
        Box::new(BinaryComparison {
            left: left,
            operator: operator,
            right: right,
        }),
    ))
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
    fn test_comparing_numbers() {
        let (context, record) = empty_context_and_record();

        let result = parse_binary_comparison("1 < 2");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parse_binary_comparison("2 < 1");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0)),
        );
    }

    #[test]
    fn test_comparing_strings() {
        let (context, record) = empty_context_and_record();

        let result = parse_binary_comparison(r#""a" < "b""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parse_binary_comparison(r#""A" < "a""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );
    }

    #[test]
    fn test_comparing_numbers_and_strings() {
        let (context, record) = empty_context_and_record();

        // Numbers come before letters
        let result = parse_binary_comparison(r#""a" < 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0)),
        );
    }
}
