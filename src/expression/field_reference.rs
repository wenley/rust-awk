use regex::Regex;
use std::fmt::Debug;

use nom::{
    character::complete::{multispace0, one_of},
    multi::many0,
    sequence::{pair, terminated},
};

use super::{Expression, ExpressionParseResult};
use crate::{
    basic_types::{Context, Record},
    function::Functions,
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

    fn evaluate<'a>(
        &self,
        functions: &Functions,
        context: &mut Context,
        record: &'a Record,
    ) -> Value {
        let value = self
            .expression
            .evaluate(functions, context, record)
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

pub(super) fn field_reference_parser<F>(next_parser: F) -> impl Fn(&str) -> ExpressionParseResult
where
    F: Fn(&str) -> ExpressionParseResult,
{
    move |input: &str| {
        let (i, (references, inner_expression)) =
            pair(many0(terminated(one_of("$"), multispace0)), |i| {
                next_parser(i)
            })(input)?;
        let expression = references.iter().fold(inner_expression, |inner, _| {
            Box::new(FieldReference { expression: inner })
        });
        Result::Ok((i, expression))
    }
}

#[cfg(test)]
mod tests {
    use super::super::literal::*;
    use super::*;
    use crate::function::Functions;
    use std::collections::HashMap;

    fn empty_context_and_record() -> (Functions, Context, Record<'static>) {
        (
            HashMap::new(),
            Context::empty(),
            Record {
                full_line: "",
                fields: vec![],
            },
        )
    }

    #[test]
    fn field_reference_can_evaluate() {
        let (functions, mut context, mut record) = empty_context_and_record();
        record.fields = vec!["first", "second"];

        assert_eq!(
            FieldReference {
                expression: Box::new(Literal::Numeric(NumericValue::Integer(1)))
            }
            .evaluate(&functions, &mut context, &record),
            Value::String("first".to_string()),
        );
    }

    #[test]
    fn test_parse_field_reference() {
        let (functions, mut context, mut record) = empty_context_and_record();
        let parser = field_reference_parser(parse_literal);

        let result = parser("$1");
        assert_eq!(result.is_ok(), true);
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&functions, &mut context, &record),
            Value::Uninitialized,
        );

        record.fields = vec!["hello"];
        assert_eq!(
            expression.evaluate(&functions, &mut context, &record),
            Value::String("hello".to_string()),
        );

        let result = parser("$     1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result
                .unwrap()
                .1
                .evaluate(&functions, &mut context, &record),
            Value::String("hello".to_string()),
        );
    }

    // #[test]
    // fn test_nested_field_references() {
    //     let (functions, mut context, mut record) = empty_context_and_record();
    //     record.fields = vec!["2", "3", "hello"];

    //     let parser = field_reference_parser(parse_literal);
    //     let result = parser("$$$1");
    //     assert!(result.is_ok(), true);
    //     let expression = result.unwrap().1;
    //     assert_eq!(
    //         expression.evaluate(&functions, &mut context, &record),
    //         Value::String("hello".to_string()),
    //     );
    // }
}
