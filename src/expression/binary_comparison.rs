use regex::Regex;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::map,
    sequence::{delimited, tuple},
};

use super::{Expression, ExpressionParseResult};
use crate::{
    basic_types::MutableContext,
    function::Functions,
    value::{NumericValue, Value},
};

#[derive(Debug)]
enum Operator {
    Less,
    LessEqual,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
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

    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Value {
        let left_value = self.left.evaluate(functions, context);
        let right_value = self.right.evaluate(functions, context);

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
                Operator::LessEqual => x <= y,
                Operator::Equal => x == y,
                Operator::NotEqual => x != y,
                Operator::Greater => x > y,
                Operator::GreaterEqual => x >= y,
            }
        } else {
            let (s1, s2) = (
                left_value.coerce_to_string(),
                right_value.coerce_to_string(),
            );
            match &self.operator {
                Operator::Less => s1 < s2,
                Operator::LessEqual => s1 <= s2,
                Operator::Equal => s1 == s2,
                Operator::NotEqual => s1 != s2,
                Operator::Greater => s1 > s2,
                Operator::GreaterEqual => s1 >= s2,
            }
        };

        let int_value = if result { 1 } else { 0 };
        Value::Numeric(NumericValue::Integer(int_value))
    }
}

pub(super) fn comparison_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        alt((definite_comparison_parser(|i| next_parser(i)), |i| {
            next_parser(i)
        }))(input)
    }
}

fn definite_comparison_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        let operators = alt((
            // Need to check longer tags before shorter tags due to prefix collision
            tag("<="),
            tag("=="),
            tag("!="),
            tag(">="),
            tag(">"),
            tag("<"),
        ));
        let parse_operator =
            map(
                delimited(multispace0, operators, multispace0),
                |operator| match operator {
                    "<" => Operator::Less,
                    "<=" => Operator::LessEqual,
                    "==" => Operator::Equal,
                    "!=" => Operator::NotEqual,
                    ">" => Operator::Greater,
                    ">=" => Operator::GreaterEqual,
                    _ => panic!("Unrecognized comparison character: {}", operator),
                },
            );

        let (i, (left, operator, right)) =
            tuple((|i| next_parser(i), parse_operator, |i| next_parser(i)))(input)?;

        Result::Ok((
            i,
            Box::new(BinaryComparison {
                left: left,
                operator: operator,
                right: right,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::literal::*;
    use super::*;
    use crate::basic_types::Variables;
    use crate::function::Functions;
    use std::collections::HashMap;

    fn empty_functions_and_variables() -> (Functions, Variables) {
        let variables = Variables::empty();
        (HashMap::new(), variables)
    }

    #[test]
    fn test_comparing_numbers() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

        let parser = comparison_parser(parse_literal);

        let result = parser("1 < 2");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser("1 > 2");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(0)),
        );
    }

    #[test]
    fn test_comparing_strings() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

        let parser = comparison_parser(parse_literal);

        let result = parser(r#""a" < "b""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser(r#""A" <= "a""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(1)),
        );
    }

    #[test]
    fn test_comparing_numbers_and_strings() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

        let parser = comparison_parser(parse_literal);

        // Numbers come before letters
        let result = parser(r#""a" < 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(0)),
        );

        let result = parser(r#""1" == 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(1)),
        );
    }
}
