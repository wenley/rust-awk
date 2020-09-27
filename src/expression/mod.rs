use regex::Regex;
use std::fmt::Debug;

use nom::{
    branch::alt,
    character::complete::{multispace0, one_of},
    combinator::map,
    sequence::tuple,
    IResult,
};

use crate::{basic_types::MutableContext, function::Functions, value::Value};

mod binary_comparison;
mod binary_math;
mod boolean;
mod field_reference;
mod function;
mod literal;
mod regex_match;
pub(crate) mod variable;

pub(crate) use variable::parse_variable_name;

pub(crate) trait Expression: Debug {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Value;

    fn regex<'a>(&'a self) -> Option<&'a Regex>;
}

pub(crate) trait Assign: Debug {
    fn assign<'a>(&self, context: &mut MutableContext, value: Value);
}

type ExpressionParseResult<'a> = IResult<&'a str, Box<dyn Expression>>;

/// Tiers of parsing
///
/// The top-level parser is responsible for the loosest-binding / lowest-precedence
/// operators. As we descend the levels, we encounter tighter-binding operators
/// until we reach literals and the parenthesized expressions.

pub(crate) fn parse_assignable(input: &str) -> IResult<&str, Box<dyn Assign>> {
    variable::parse_assignable_variable(input)
}

pub(crate) fn parse_expression(input: &str) -> ExpressionParseResult {
    // Descending order of precedence
    let field_reference_parser = field_reference::field_reference_parser(parse_primary);
    let not_parser = boolean::not_parser(field_reference_parser);
    let multiplication_parser = binary_math::multiplication_parser(not_parser);
    let addition_parser = binary_math::addition_parser(multiplication_parser);
    let comparison_parser = binary_comparison::comparison_parser(addition_parser);
    let regex_parser = regex_match::regex_parser(comparison_parser);
    let and_parser = boolean::and_parser(regex_parser);
    let or_parser = boolean::or_parser(and_parser);

    or_parser(input)
}

fn parse_primary(input: &str) -> ExpressionParseResult {
    alt((
        function::parse_function_call,
        literal::parse_literal,
        variable::parse_variable,
        parse_parens,
    ))(input)
}

fn parse_parens(input: &str) -> ExpressionParseResult {
    map(
        tuple((
            one_of("("),
            multispace0,
            parse_expression,
            multispace0,
            one_of(")"),
        )),
        |(_, _, expression, _, _)| expression,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::{Record, Variables};
    use crate::function::Functions;
    use crate::value::NumericValue;
    use std::collections::HashMap;

    fn empty_variables_and_record() -> (Functions, Variables, Record<'static>) {
        let variables = Variables::empty();
        let record = variables.record_for_line("");
        (HashMap::new(), variables, record)
    }

    #[test]
    fn test_parse_parens() {
        let (functions, mut variables, record) = empty_variables_and_record();
        let mut context = MutableContext {
            variables: &mut variables,
            record: Some(&record),
        };

        let result = parse_expression("( 1 )");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(1))
        );

        let result = parse_expression("(1) + (2.5)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Float(3.5))
        );
    }

    #[test]
    fn test_boolean_precedence() {
        let (functions, mut variables, record) = empty_variables_and_record();
        let mut context = MutableContext {
            variables: &mut variables,
            record: Some(&record),
        };

        let result = parse_expression("1 && 1 || 0 && 1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(1))
        );

        let result = parse_expression("1 && 0 || 0 && 1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(0))
        );

        let result = parse_expression("0 || 1 && 0 || 1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context),
            Value::Numeric(NumericValue::Integer(1))
        );
    }
}
