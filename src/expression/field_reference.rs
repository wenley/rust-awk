use regex::Regex;
use std::fmt::Debug;

use nom::{
    character::complete::{multispace0, one_of},
    multi::many0,
    sequence::{pair, terminated},
};

use super::{Expression, ExpressionParseResult};
use crate::{
    basic_types::MutableContext,
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

    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Value {
        let value = self
            .expression
            .evaluate(functions, context)
            .coerce_to_numeric();
        let unsafe_index = match value {
            NumericValue::Integer(i) => i,
            NumericValue::Float(f) => f.floor() as i64,
        };
        context.fetch_field(unsafe_index)
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
    use crate::basic_types::{Record, Variables};
    use crate::function::Functions;
    use std::collections::HashMap;

    fn empty_variables_and_record() -> (Functions, Variables, Record<'static>) {
        let variables = Variables::empty();
        let record = variables.record_for_line("");
        (HashMap::new(), variables, record)
    }

    #[test]
    fn field_reference_can_evaluate() {
        let (functions, mut variables, _) = empty_variables_and_record();
        let record = variables.record_for_line("first second");
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record(&record);

        assert_eq!(
            FieldReference {
                expression: Box::new(Literal::Numeric(NumericValue::Integer(1)))
            }
            .evaluate(&functions, &mut context),
            Value::String("first".to_string()),
        );
    }

    #[test]
    fn test_parse_field_reference() {
        let (functions, mut variables, mut record) = empty_variables_and_record();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record(&record);

        let parser = field_reference_parser(parse_literal);

        let result = parser("$1");
        assert_eq!(result.is_ok(), true);
        let expression = result.unwrap().1;
        assert_eq!(
            expression.evaluate(&functions, &mut context),
            Value::Uninitialized,
        );

        record = variables.record_for_line("hello");
        context = MutableContext::for_variables(&mut variables);
        context.set_record(&record);
        assert_eq!(
            expression.evaluate(&functions, &mut context),
            Value::String("hello".to_string()),
        );

        let result = parser("$     1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::String("hello".to_string()),
        );
    }

    // #[test]
    // fn test_nested_field_references() {
    //     let (functions, mut variables, mut record) = empty_variables_and_record();
    //     let mut context = MutableContext::for_variables(&mut variables);
    //     context.set_record(&record);
    //     record.fields = vec!["2", "3", "hello"];

    //     let parser = field_reference_parser(parse_literal);
    //     let result = parser("$$$1");
    //     assert!(result.is_ok(), true);
    //     let expression = result.unwrap().1;
    //     assert_eq!(
    //         expression.evaluate(&functions, &mut context),
    //         Value::String("hello".to_string()),
    //     );
    // }
}
