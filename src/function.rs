use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1, one_of},
    combinator::map,
    multi::many0,
    sequence::{pair, preceded, tuple},
    IResult,
};
use std::collections::HashMap;

use crate::{
    action::{parse_action, Action},
    expression::variable::parse_variable_name,
    value::Value,
};

pub(crate) struct StackFrame {
    variables: HashMap<String, Value>,
}

impl StackFrame {
    pub(crate) fn empty() -> StackFrame {
        StackFrame {
            variables: HashMap::new(),
        }
    }

    pub(crate) fn fetch_variable(&self, variable_name: &str) -> Option<Value> {
        self.variables.get(variable_name).map(|val| val.clone())
    }

    pub(crate) fn assign_variable(&mut self, variable_name: &str, value: Value) {
        self.variables.insert(variable_name.to_string(), value);
    }
}

pub(crate) struct FunctionDefinition {
    pub(crate) name: String,
    pub(crate) variable_names: Vec<String>,
    body: Action,
}

pub(crate) fn parse_function(input: &str) -> IResult<&str, FunctionDefinition> {
    let parse_variable_list = map(
        pair(
            parse_variable_name,
            many0(preceded(
                tuple((multispace0, one_of(","), multispace0)),
                parse_variable_name,
            )),
        ),
        |(name, mut names)| {
            names.insert(0, name);
            names
        },
    );
    map(
        tuple((
            tag("function"),
            multispace1,
            parse_variable_name,
            multispace0,
            tag("("),
            multispace0,
            parse_variable_list,
            multispace0,
            tag(")"),
            multispace0,
            parse_action,
        )),
        |(_, _, func_name, _, _, _, variables, _, _, _, body)| FunctionDefinition {
            name: func_name.to_string(),
            variable_names: variables.iter().map(|s| s.to_string()).collect(),
            body: body,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_function() {
        let result = parse_function(
            r#"function foo(a) {
                print("hi");
            }"#,
        );
        assert!(result.is_ok());
        let function_definition = result.unwrap().1;
        assert_eq!(function_definition.name, "foo");
        assert_eq!(function_definition.variable_names, vec!["a"]);
    }
}
