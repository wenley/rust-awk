use regex::Regex;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{pair, tuple},
};

use super::{AssignableExpression, Expression, ExpressionParseResult};
use crate::{
    context::MutableContext,
    function::Functions,
    printable::Printable,
    value::{NumericValue, Value},
};

#[derive(Debug)]
enum IncrementType {
    Prefix,
    Postfix,
}

#[derive(Debug)]
struct Increment {
    variable: Box<dyn AssignableExpression>,
    increment_type: IncrementType,
    is_increment: bool, // true for ++, false for --
}

impl Expression for Increment {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<Value> {
        // Get current value
        let current_value = self.variable.evaluate(functions, context).value;

        // Calculate new value
        let new_value = match current_value.coerce_to_numeric() {
            NumericValue::Integer(i) => {
                let delta = if self.is_increment { 1 } else { -1 };
                Value::Numeric(NumericValue::Integer(i + delta))
            }
            NumericValue::Float(f) => {
                let delta = if self.is_increment { 1.0 } else { -1.0 };
                Value::Numeric(NumericValue::Float(f + delta))
            }
        };

        // Assign new value
        self.variable.assign(functions, context, new_value.clone());

        // Return appropriate value based on prefix/postfix
        match self.increment_type {
            IncrementType::Prefix => Printable::wrap(new_value),
            IncrementType::Postfix => Printable::wrap(current_value),
        }
    }
}

fn parse_prefix_increment(input: &str) -> ExpressionParseResult {
    map(
        tuple((
            alt((tag("++"), tag("--"))),
            super::parse_assignable,
        )),
        |(op, variable)| {
            Box::new(Increment {
                variable: variable,
                increment_type: IncrementType::Prefix,
                is_increment: op == "++",
            }) as Box<dyn Expression>
        },
    )(input)
}

fn parse_postfix_increment(input: &str) -> ExpressionParseResult {
    map(
        pair(
            super::parse_assignable,
            alt((tag("++"), tag("--"))),
        ),
        |(variable, op)| {
            Box::new(Increment {
                variable: variable,
                increment_type: IncrementType::Postfix,
                is_increment: op == "++",
            }) as Box<dyn Expression>
        },
    )(input)
}

pub(super) fn increment_decrement_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        // Try prefix increment/decrement first
        if let Ok(result) = parse_prefix_increment(input) {
            return Ok(result);
        }
        // Try postfix increment/decrement next
        if let Ok(result) = parse_postfix_increment(input) {
            return Ok(result);
        }
        // If neither worked, try the next parser
        next_parser(input)
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse_expression;
    use super::*;
    use crate::test_utilities::empty_functions_and_variables;
    use crate::value::NumericValue;
    use crate::context::VariableStore;

    #[test]
    fn test_prefix_increment() {
        let (functions, mut variables) = empty_functions_and_variables();
        variables.assign_variable("x", Value::Numeric(NumericValue::Integer(5)));
        let mut context = MutableContext::for_variables(&mut variables);

        let result = parse_expression("++x");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(6))
        );
        assert_eq!(
            context.fetch_variable("x"),
            Value::Numeric(NumericValue::Integer(6))
        );
    }

    #[test]
    fn test_postfix_increment() {
        let (functions, mut variables) = empty_functions_and_variables();
        variables.assign_variable("x", Value::Numeric(NumericValue::Integer(5)));
        let mut context = MutableContext::for_variables(&mut variables);

        let result = parse_expression("x++");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(5))
        );
        assert_eq!(
            context.fetch_variable("x"),
            Value::Numeric(NumericValue::Integer(6))
        );
    }

    #[test]
    fn test_prefix_decrement() {
        let (functions, mut variables) = empty_functions_and_variables();
        variables.assign_variable("x", Value::Numeric(NumericValue::Integer(5)));
        let mut context = MutableContext::for_variables(&mut variables);

        let result = parse_expression("--x");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(4))
        );
        assert_eq!(
            context.fetch_variable("x"),
            Value::Numeric(NumericValue::Integer(4))
        );
    }

    #[test]
    fn test_postfix_decrement() {
        let (functions, mut variables) = empty_functions_and_variables();
        variables.assign_variable("x", Value::Numeric(NumericValue::Integer(5)));
        let mut context = MutableContext::for_variables(&mut variables);

        let result = parse_expression("x--");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(5))
        );
        assert_eq!(
            context.fetch_variable("x"),
            Value::Numeric(NumericValue::Integer(4))
        );
    }
} 