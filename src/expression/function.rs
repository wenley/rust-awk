use regex::Regex;
use std::fmt::Debug;

use nom::{
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::many0,
    sequence::{pair, preceded, tuple},
};

use super::{parse_expression, variable::parse_variable_name, Expression, ExpressionParseResult};
use crate::{
    basic_types::{Context, Record},
    function::Functions,
    value::Value,
};

#[derive(Debug)]
struct FunctionCall {
    name: String,
    arguments: Vec<Box<dyn Expression>>,
}

impl Expression for FunctionCall {
    fn evaluate<'a>(
        &self,
        functions: &Functions,
        context: &mut Context,
        record: &'a Record,
    ) -> Value {
        let function = match functions.get(&self.name) {
            Some(func) => func,
            None => panic!("Could not find function with name {}", self.name),
        };
        let values: Vec<Value> = self
            .arguments
            .iter()
            .map(|exp| exp.evaluate(functions, context, record))
            .collect();

        // TODO: Capture this output
        function.invoke_with(values, functions, context, record);

        // TODO: Actually return a proper return value
        Value::String("".to_string())
    }

    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }
}

pub(crate) fn parse_function_call(input: &str) -> ExpressionParseResult {
    let parse_arguments = map(
        pair(
            parse_expression,
            many0(preceded(
                tuple((multispace0, one_of(","), multispace0)),
                parse_expression,
            )),
        ),
        |(argument, mut arguments)| {
            arguments.insert(0, argument);
            arguments
        },
    );

    let (i, (func_name, _, _, _, arguments, _, _)) = tuple((
        parse_variable_name,
        multispace0,
        one_of("("),
        multispace0,
        parse_arguments,
        multispace0,
        one_of(")"),
    ))(input)?;

    Result::Ok((
        i,
        Box::new(FunctionCall {
            name: func_name.to_string(),
            arguments: arguments,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_parsing() {
        // Assert no panic
        let result = parse_function_call(r#"foo("first", a, 1 + 2, $0)"#);
        assert!(result.is_ok());
        let (remaining, call) = result.unwrap();
        assert_eq!(remaining, "");
    }
}
