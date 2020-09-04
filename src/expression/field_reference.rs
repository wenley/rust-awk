use regex::Regex;
use std::fmt::Debug;

use nom::{character::complete::one_of, sequence::preceded};

use super::{Expression, ExpressionParseResult};
use crate::{
    basic_types::{Context, Record},
    value::{NumericValue, Value},
};

#[derive(Debug)]
struct FieldReference {
    expression: Box<dyn Expression>,
}

impl Expression for FieldReference {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        let value = self
            .expression
            .evaluate(context, record)
            .coerce_to_numeric();
        let unsafe_index = match value {
            NumericValue::Integer(i) => i,
            NumericValue::Float(f) => f.floor() as i64,
        };
        match unsafe_index {
            i if i < 0 => panic!("Field indexes cannot be negative: {}", unsafe_index),
            i if i == 0 => Value::String(record.full_line.to_string()),
            i => record
                .fields
                .get((i - 1) as usize)
                .map(|s| Value::String(s.to_string()))
                .unwrap_or(Value::Uninitialized),
        }
    }
}

pub(super) fn parse_field_reference(input: &str) -> ExpressionParseResult {
    let (i, expr) = preceded(one_of("$"), super::parse_expression)(input)?;
    Result::Ok((i, Box::new(FieldReference { expression: expr })))
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
    fn field_reference_can_evaluate() {
        let (context, mut record) = empty_context_and_record();
        record.fields = vec!["first", "second"];

        assert_eq!(
            FieldReference {
                expression: Box::new(Literal::Numeric(NumericValue::Integer(1)))
            }
            .evaluate(&context, &record),
            Value::String("first".to_string()),
        );
    }

    #[test]
    fn test_parse_field_reference() {
        let (context, mut record) = empty_context_and_record();
        let result = parse_field_reference("$1");
        assert_eq!(result.is_ok(), true);
        let expression = result.unwrap().1;
        assert_eq!(expression.evaluate(&context, &record), Value::Uninitialized,);

        record.fields = vec!["hello"];
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::String("hello".to_string()),
        );
    }
}
