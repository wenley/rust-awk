use regex::Regex;
use std::fmt::Debug;

use nom::{branch::alt, character::complete::one_of, sequence::delimited, IResult};

use crate::{
    basic_types::{Context, Record},
    value::Value,
};

mod binary_math;
mod field_reference;
mod literal;
mod regex_match;
mod variable;

pub(crate) trait Expression: Debug {
    fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value;

    fn regex<'a>(&'a self) -> Option<&'a Regex>;
}

pub(crate) trait Assign: Debug {
    fn assign<'a>(&self, context: &mut Context, record: &'a Record, value: Value);
}

/// Tiers of parsing
///
/// The top-level parser is responsible for the loosest-binding / lowest-precedence
/// operators. As we descend the levels, we encounter tighter-binding operators
/// until we reach literals and the parenthesized expressions.

pub(crate) fn parse_assignable(input: &str) -> IResult<&str, Box<dyn Assign>> {
    variable::parse_assignable_variable(input)
}

pub(crate) fn parse_expression(input: &str) -> IResult<&str, Box<dyn Expression>> {
    alt((regex_match::parse_regex_match, binary_math::parse_addition))(input)
}

fn parse_primary(input: &str) -> IResult<&str, Box<dyn Expression>> {
    alt((
        literal::parse_literal,
        variable::parse_variable,
        parse_parens,
        field_reference::parse_field_reference,
    ))(input)
}

fn parse_parens(input: &str) -> IResult<&str, Box<dyn Expression>> {
    delimited(one_of("("), parse_expression, one_of(")"))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::NumericValue;

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
    fn test_parse_parens() {
        let (context, record) = empty_context_and_record();

        let result = parse_expression("(1)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1))
        );

        let result = parse_expression("(1) + (2.5)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&context, &record),
            Value::Numeric(NumericValue::Float(3.5))
        );
    }
}
