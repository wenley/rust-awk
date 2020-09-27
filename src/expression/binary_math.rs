use regex::Regex;

use nom::{
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::many0,
    sequence::{delimited, pair},
};

use super::{Expression, ExpressionParseResult};
use crate::{
    basic_types::MutableContext,
    function::Functions,
    value::{NumericValue, Value},
};

#[derive(Debug)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    // Exponent,
}

#[derive(Debug)]
struct BinaryMath {
    left: Box<dyn Expression>,
    operator: Operator,
    right: Box<dyn Expression>,
}

impl Expression for BinaryMath {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Value {
        let left_value = self.left.evaluate(functions, context).coerce_to_numeric();
        let right_value = self.right.evaluate(functions, context).coerce_to_numeric();

        match (&self.operator, left_value, right_value) {
            (Operator::Add, NumericValue::Integer(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Integer(x + y))
            }
            (Operator::Add, NumericValue::Integer(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float((x as f64) + y))
            }
            (Operator::Add, NumericValue::Float(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Float(x + (y as f64)))
            }
            (Operator::Add, NumericValue::Float(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float(x + y))
            }
            (Operator::Subtract, NumericValue::Integer(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Integer(x - y))
            }
            (Operator::Subtract, NumericValue::Integer(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float((x as f64) - y))
            }
            (Operator::Subtract, NumericValue::Float(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Float(x - (y as f64)))
            }
            (Operator::Subtract, NumericValue::Float(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float(x - y))
            }
            (Operator::Multiply, NumericValue::Integer(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Integer(x * y))
            }
            (Operator::Multiply, NumericValue::Integer(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float((x as f64) * y))
            }
            (Operator::Multiply, NumericValue::Float(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Float(x * (y as f64)))
            }
            (Operator::Multiply, NumericValue::Float(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float(x * y))
            }
            (Operator::Divide, NumericValue::Integer(x), NumericValue::Integer(y)) => {
                if x % y == 0 {
                    Value::Numeric(NumericValue::Integer(x / y))
                } else {
                    // When y does not divide x, Awk switches to floating point division
                    Value::Numeric(NumericValue::Float((x as f64) / (y as f64)))
                }
            }
            (Operator::Divide, NumericValue::Integer(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float((x as f64) / y))
            }
            (Operator::Divide, NumericValue::Float(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Float(x / (y as f64)))
            }
            (Operator::Divide, NumericValue::Float(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float(x / y))
            }
            (Operator::Modulo, NumericValue::Integer(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Integer(x % y))
            }
            (Operator::Modulo, NumericValue::Integer(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float((x as f64) % y))
            }
            (Operator::Modulo, NumericValue::Float(x), NumericValue::Integer(y)) => {
                Value::Numeric(NumericValue::Float(x % (y as f64)))
            }
            (Operator::Modulo, NumericValue::Float(x), NumericValue::Float(y)) => {
                Value::Numeric(NumericValue::Float(x % y))
            }
        }
    }
}

pub(super) fn addition_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        let parse_added_expr = pair(
            map(
                delimited(multispace0, one_of("+-"), multispace0),
                |operator_char| match operator_char {
                    '+' => Operator::Add,
                    '-' => Operator::Subtract,
                    _ => panic!("Unrecognized binary math operator {}", operator_char),
                },
            ),
            |i| next_parser(i),
        );
        // Why does this `map` work??
        map(
            pair(|i| next_parser(i), many0(parse_added_expr)),
            move |(first, mut rest)| {
                rest.drain(0..).fold(first, |inner, (operator, next)| {
                    Box::new(BinaryMath {
                        left: inner,
                        operator: operator,
                        right: next,
                    })
                })
            },
        )(input)
    }
}

// Since multiplication is a higher precedence, it is lower level -> gets to consume characters
// first
pub(super) fn multiplication_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        let parse_added_expr = pair(
            map(
                delimited(multispace0, one_of("*/%"), multispace0),
                |operator_char| match operator_char {
                    '*' => Operator::Multiply,
                    '/' => Operator::Divide,
                    '%' => Operator::Modulo,
                    _ => panic!("Unrecognized binary math operator {}", operator_char),
                },
            ),
            |i| next_parser(i),
        );
        map(
            pair(|i| next_parser(i), many0(parse_added_expr)),
            move |(first, mut rest)| {
                rest.drain(0..).fold(first, |inner, (operator, next)| {
                    Box::new(BinaryMath {
                        left: inner,
                        operator: operator,
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
    use crate::basic_types::{Record, Variables};
    use crate::function::Functions;
    use std::collections::HashMap;

    fn empty_variables_and_record() -> (Functions, Variables, Record<'static>) {
        (
            HashMap::new(),
            Variables::empty(),
            Record {
                full_line: "",
                fields: vec![],
            },
        )
    }

    #[test]
    fn binary_expressions_can_evaluate() {
        let (functions, mut variables, record) = empty_variables_and_record();
        let mut context = MutableContext {
            variables: &mut variables,
            record: &record,
        };
        assert_eq!(
            BinaryMath {
                left: Box::new(Literal::Numeric(NumericValue::Integer(2))),
                operator: Operator::Add,
                right: Box::new(Literal::Numeric(NumericValue::Integer(3))),
            }
            .evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(5)),
        );
    }

    #[test]
    fn binary_expressions_can_parse() {
        let (functions, mut variables, record) = empty_variables_and_record();
        let mut context = MutableContext {
            variables: &mut variables,
            record: &record,
        };
        let parser = addition_parser(multiplication_parser(parse_literal));

        let result = parser("1 + 2 - 3 + 4 - 5.5");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Float(-1.5)),
        );

        let result = parser("1 * 2 + 3 * 4");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(14)),
        );

        let result = parser("6 / 5 * 4 / 3");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            // Floating point error!
            Value::Numeric(NumericValue::Float(1.5999999999999999)),
        );

        let result = parser("6 / 3");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(2)),
        );

        let result = parser("6 % 5 * 4 / 3 % 2");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Float(1.3333333333333333)),
        );
    }
}
