use regex::Regex;

use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::{many0, many0_count},
    sequence::{delimited, pair, preceded, terminated},
};

use super::{Expression, ExpressionParseResult};
use crate::{
    context::MutableContext,
    function::Functions,
    printable::Printable,
    value::{NumericValue, Value},
};

#[derive(Debug)]
enum Operator {
    And,
    Or,
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

    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<Value> {
        let Printable {
            value: left_value,
            output: mut left_output,
        } = self.left.evaluate(functions, context);
        let Printable {
            value: right_value,
            output: mut right_output,
        } = self.right.evaluate(functions, context);

        let (left, right) = (
            left_value.coercion_to_boolean(),
            right_value.coercion_to_boolean(),
        );

        let result = match &self.operator {
            Operator::And => left && right,
            Operator::Or => left || right,
        };
        let int_value = if result { 1 } else { 0 };
        left_output.append(&mut right_output);
        Printable {
            value: Value::Numeric(NumericValue::Integer(int_value)),
            output: left_output,
        }
    }
}

#[derive(Debug)]
struct NotBoolean {
    expression: Box<dyn Expression>,
}

impl Expression for NotBoolean {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<Value> {
        self.expression.evaluate(functions, context).map(|value| {
            let int_value = if value.coercion_to_boolean() { 0 } else { 1 };
            Value::Numeric(NumericValue::Integer(int_value))
        })
    }
}

pub(super) fn not_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        let (i, (nestings, inner_expression)) =
            pair(many0_count(terminated(one_of("!"), multispace0)), |i| {
                next_parser(i)
            })(input)?;

        // Collapse nested negations for efficiency
        let expression = if nestings == 0 {
            inner_expression
        } else if nestings % 2 == 1 {
            Box::new(NotBoolean {
                expression: inner_expression,
            })
        } else {
            // Still need to coerce to {0,1} if the value was something else
            Box::new(NotBoolean {
                expression: Box::new(NotBoolean {
                    expression: inner_expression,
                }),
            })
        };
        Result::Ok((i, expression))
    }
}

pub(super) fn or_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        let parse_added_expr = preceded(delimited(multispace0, tag("||"), multispace0), |i| {
            next_parser(i)
        });
        map(
            pair(|i| next_parser(i), many0(parse_added_expr)),
            move |(first, mut rest)| {
                rest.drain(0..).fold(first, |inner, next| {
                    Box::new(BinaryBoolean {
                        left: inner,
                        operator: Operator::Or,
                        right: next,
                    })
                })
            },
        )(input)
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
    use crate::test_utilities::empty_functions_and_variables;

    #[test]
    fn test_and_parsing() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        let parser = and_parser(parse_literal);

        let result = parser(r#""a" && 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser(r#""a" && 0"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(0)),
        );

        let result = parser(r#""" && 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(0)),
        );
    }

    #[test]
    fn test_or_parsing() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        let parser = or_parser(parse_literal);

        let result = parser(r#""a" || 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser(r#""a" || 0"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser(r#""" || 1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser(r#""" || 0"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(0)),
        );
    }

    #[test]
    fn test_not_parsing() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        let parser = not_parser(parse_literal);

        let result = parser(r#"!1"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(0)),
        );

        let result = parser(r#"!0"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser(r#"!"a""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(0)),
        );

        let result = parser(r#"!"""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parser(r#"!!!!!0"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );
    }

    #[test]
    fn test_iteration_compression() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        let parser = not_parser(parse_literal);

        let result = parser(r#""abc""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::String("abc".to_string()),
        );

        let result = parser(r#"!!"abc""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).value,
            Value::Numeric(NumericValue::Integer(1)),
        );
    }
}
