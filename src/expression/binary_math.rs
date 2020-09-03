use regex::Regex;

use nom::{
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::many0,
    sequence::{delimited, pair},
    IResult,
};

use super::Expression;
use crate::{
    basic_types::{Context, Record},
    value::{NumericValue, Value},
};

#[derive(Debug)]
enum Operator {
    Add,
    Subtract,
    // Multiply,
    // Divide,
    // Modulo,
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

    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        let left_value = self.left.evaluate(context, record).coerce_to_numeric();
        let right_value = self.right.evaluate(context, record).coerce_to_numeric();

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
            _ => panic!("Unsupported evaluation"),
        }
    }
}

pub(super) fn parse_binary_math_expression(input: &str) -> IResult<&str, Box<dyn Expression>> {
    parse_addition(input)
}

fn parse_addition(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let parse_added_expr = pair(
        map(
            delimited(multispace0, one_of("+-"), multispace0),
            |operator_char| match operator_char {
                '+' => Operator::Add,
                '-' => Operator::Subtract,
                _ => panic!("Unrecognized binary math operator {}", operator_char),
            },
        ),
        super::parse_primary,
    );
    // Why does this `map` work??
    map(
        pair(super::parse_primary, many0(parse_added_expr)),
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
    fn binary_expressions_can_evaluate() {
        let (context, record) = empty_context_and_record();
        assert_eq!(
            BinaryMath {
                left: Box::new(Literal::Numeric(NumericValue::Integer(2))),
                operator: Operator::Add,
                right: Box::new(Literal::Numeric(NumericValue::Integer(3))),
            }
            .evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(5)),
        );
    }
}
