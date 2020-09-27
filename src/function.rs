use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1, one_of},
    combinator::map,
    multi::many0,
    sequence::{pair, preceded, tuple},
    IResult,
};
use std::collections::HashMap;
use std::ops::Index;

use crate::{
    action::{parse_action, Action},
    basic_types::{Record, Variables, UNINITIALIZED_VALUE},
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

pub(crate) type Functions = HashMap<String, FunctionDefinition>;

impl FunctionDefinition {
    pub(crate) fn invoke_with(
        &self,
        values: Vec<Value>,
        functions: &Functions,
        variables: &mut Variables,
        record: &Record,
    ) -> Vec<String> {
        let (num, expected_num) = (values.len(), self.variable_names.len());
        if num > expected_num {
            panic!(
                "function {} called with {} args, uses only {}",
                self.name, num, expected_num
            );
        }

        let mut frame = StackFrame::empty();
        for i in 0..values.len() {
            let (name, value) = (self.variable_names.index(i), values[i].clone());
            frame.assign_variable(name, value);
        }
        for i in values.len()..self.variable_names.len() {
            frame.assign_variable(self.variable_names.index(i), UNINITIALIZED_VALUE.clone());
        }

        // TODO: Make Functions expressions too
        // Right now, a function can only be invoked as a Statement with printable outputs.
        // In the future, a function will need to be both a "statement" (returning outputs) AND an
        // expression (having a nestable value)
        variables.with_stack_frame(frame, |c| self.body.output_for_line(functions, c, record))
    }
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
        let (remaining, function_definition) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(function_definition.name, "foo");
        assert_eq!(function_definition.variable_names, vec!["a"]);
    }
}
