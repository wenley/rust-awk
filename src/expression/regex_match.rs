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
    basic_types::MutableContext,
    function::Functions,
    printable::Printable,
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

    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<Value> {
        let Printable {
            value: left_value,
            output: mut left_output,
        } = self.left.evaluate(functions, context);
        let left_string = left_value.coerce_to_string();

        let matches = match self.right.regex() {
            Some(r) => r.is_match(&left_string),
            None => {
                let Printable {
                    value: right_value,
                    output: mut right_output,
                } = self.right.evaluate(functions, context);
                let right_string = right_value.coerce_to_string();
                left_output.append(&mut right_output);
                Regex::new(&right_string).unwrap().is_match(&left_string)
            }
        };
        let int_value = if matches ^ self.negated { 1 } else { 0 };

        Printable {
            value: Value::Numeric(NumericValue::Integer(int_value)),
            output: left_output,
        }
    }
}

pub(super) fn regex_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        alt((definite_regex_parser(|i| next_parser(i)), |i| {
            next_parser(i)
        }))(input)
    }
}

fn definite_regex_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
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
    use super::super::{binary_math::addition_parser, literal::parse_literal};
    use super::*;
    use crate::basic_types::Variables;
    use crate::function::Functions;
    use std::collections::HashMap;

    fn empty_functions_and_variables() -> (Functions, Variables) {
        let variables = Variables::empty();
        (HashMap::new(), variables)
    }

    #[test]
    fn test_regex_match() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        let parser = regex_parser(addition_parser(parse_literal));

        let result = parser("1 ~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(0)),
        );

        // Cannot consume the full expression
        let result = parser("1 ~ 2 ~ 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, " ~ 3");

        let result = parser("1 + 2 ~ 3");
        assert!(result.is_ok());
        let (remainder, expression) = result.unwrap();
        assert_eq!(remainder, "");
        assert_eq!(
            expression.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser("1 !~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );
    }
}
