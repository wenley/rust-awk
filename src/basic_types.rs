use crate::value::Value;
use regex;
use std::collections::HashMap;
use std::ops::Index;

pub(crate) struct Record<'a> {
    pub(crate) full_line: &'a str,
    pub(crate) fields: Vec<&'a str>,
}

static UNINITIALIZED_VALUE: Value = Value::Uninitialized;

enum FieldSeparator {
    Character(char),
    Regex(regex::Regex),
}

pub(crate) struct Context {
    field_separator: FieldSeparator,
    global_variables: StackFrame,
    functions: HashMap<String, FunctionSignature>,
    function_variables: Vec<StackFrame>,
}

struct StackFrame {
    variables: HashMap<String, Value>,
}

struct FunctionSignature {
    variable_names: Vec<String>,
}

impl StackFrame {
    fn empty() -> StackFrame {
        StackFrame {
            variables: HashMap::new(),
        }
    }

    fn fetch_variable(&self, variable_name: &str) -> Option<Value> {
        self.variables.get(variable_name).map(|val| val.clone())
    }

    fn assign_variable(&mut self, variable_name: &str, value: Value) {
        self.variables.insert(variable_name.to_string(), value);
    }
}

impl Context {
    pub(crate) fn empty() -> Context {
        Context {
            field_separator: FieldSeparator::Character(' '),
            functions: HashMap::new(),
            global_variables: StackFrame::empty(),
            function_variables: vec![],
        }
    }

    pub(crate) fn fetch_variable(&self, variable_name: &str) -> Value {
        let last_frame = self.function_variables.last();

        last_frame
            .and_then(|frame| frame.fetch_variable(variable_name))
            .or_else(|| self.global_variables.fetch_variable(variable_name))
            .unwrap_or_else(|| UNINITIALIZED_VALUE.clone())
    }

    pub(crate) fn set_field_separator(&mut self, new_separator: &str) {
        if new_separator.len() == 1 {
            self.field_separator = FieldSeparator::Character(new_separator.chars().next().unwrap())
        } else {
            self.field_separator = FieldSeparator::Regex(regex::Regex::new(new_separator).unwrap())
        }
    }

    pub(crate) fn assign_variable(&mut self, variable_name: &str, value: Value) {
        if let Some(frame) = self.function_variables.last_mut() {
            if let Some(_) = frame.fetch_variable(variable_name) {
                frame.assign_variable(variable_name, value);
                return;
            }
        }

        self.global_variables.assign_variable(variable_name, value);
    }

    pub(crate) fn push_stack(&mut self, function_name: &str, variables: Vec<Value>) {
        match self.functions.get(function_name) {
            None => panic!("calling undefined function {}", function_name),
            Some(FunctionSignature { variable_names }) => {
                let (num, expected_num) = (variables.len(), variable_names.len());
                if num > expected_num {
                    panic!(
                        "function {} called with {} args, uses only {}",
                        function_name, num, expected_num
                    );
                }

                let mut frame = StackFrame::empty();
                for i in 0..variables.len() {
                    let (name, value) = (variable_names.index(i), variables[i].clone());
                    frame.assign_variable(name, value);
                }
                for i in variables.len()..variable_names.len() {
                    frame.assign_variable(variable_names.index(i), UNINITIALIZED_VALUE.clone());
                }
                self.function_variables.push(frame);
            }
        }
    }

    pub(crate) fn pop_stack(&mut self) {
        self.function_variables.pop();
    }

    pub(super) fn split<'a>(&self, line: &'a str) -> Vec<&'a str> {
        match &self.field_separator {
            FieldSeparator::Character(' ') => line.split_whitespace().collect(),
            FieldSeparator::Character(c1) => line.split(|c2| *c1 == c2).collect(),
            FieldSeparator::Regex(re) => re.split(line).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::NumericValue;

    #[test]
    fn all_values_coerce_to_strings() {
        assert_eq!(
            Value::String("hello".to_string()).coerce_to_string(),
            "hello"
        );
        assert_eq!(
            Value::Numeric(NumericValue::Integer(123)).coerce_to_string(),
            "123"
        );
        assert_eq!(
            Value::Numeric(NumericValue::Float(1.234)).coerce_to_string(),
            "1.234"
        );
        assert_eq!(Value::Uninitialized.coerce_to_string(), "");
    }

    #[test]
    fn all_values_coerce_to_numerics() {
        // assert_eq!(
        //     Value::String("hello".to_string()).coerce_to_numeric(),
        //     NumericValue::Integer(0)
        // );
        assert_eq!(
            Value::Numeric(NumericValue::Integer(123)).coerce_to_numeric(),
            NumericValue::Integer(123)
        );
        assert_eq!(
            Value::Numeric(NumericValue::Float(1.234)).coerce_to_numeric(),
            NumericValue::Float(1.234)
        );
        assert_eq!(
            Value::Uninitialized.coerce_to_numeric(),
            NumericValue::Integer(0)
        );
    }

    #[test]
    fn all_values_coerce_to_booleans() {
        assert_eq!(Value::String("".to_string()).coercion_to_boolean(), false);
        assert_eq!(
            Value::String("anything".to_string()).coercion_to_boolean(),
            true
        );
        assert_eq!(
            Value::Numeric(NumericValue::Integer(0)).coercion_to_boolean(),
            false
        );
        assert_eq!(
            Value::Numeric(NumericValue::Integer(123)).coercion_to_boolean(),
            true
        );
        assert_eq!(
            Value::Numeric(NumericValue::Float(0.0)).coercion_to_boolean(),
            false
        );
        assert_eq!(
            Value::Numeric(NumericValue::Float(1.0)).coercion_to_boolean(),
            true
        );
        assert_eq!(Value::Uninitialized.coercion_to_boolean(), false);
    }

    #[test]
    fn function_variables_can_fetch() {
        let mut context = Context::empty();
        context.assign_variable("foo", Value::String("global value".to_string()));
        context.assign_variable("car", Value::String("global car".to_string()));

        let mut frame = StackFrame::empty();
        frame.assign_variable("foo", Value::String("local value".to_string()));
        context.function_variables = vec![frame];

        assert_eq!(
            context.fetch_variable("foo"),
            Value::String("local value".to_string()),
        );
        assert_eq!(
            context.fetch_variable("car"),
            Value::String("global car".to_string()),
        );
    }

    #[test]
    fn assign_during_function_assigns_global() {
        let mut context = Context::empty();
        context.function_variables = vec![StackFrame::empty()];

        context.assign_variable("foo", Value::String("value".to_string()));
        assert_eq!(
            context.fetch_variable("foo"),
            Value::String("value".to_string()),
        );
        assert_eq!(
            context.global_variables.fetch_variable("foo"),
            Some(Value::String("value".to_string())),
        );
        assert_eq!(
            context
                .function_variables
                .last()
                .unwrap()
                .fetch_variable("foo"),
            None,
        );
    }

    #[test]
    fn assign_to_function_variable_goes_local() {
        let mut context = Context::empty();
        let mut frame = StackFrame::empty();
        frame.assign_variable("foo", Value::String("old value".to_string()));

        context.function_variables = vec![frame];
        context.assign_variable("foo", Value::String("new value".to_string()));

        assert_eq!(
            context.fetch_variable("foo"),
            Value::String("new value".to_string()),
        );
        assert_eq!(context.global_variables.fetch_variable("foo"), None,);
        assert_eq!(
            context
                .function_variables
                .last()
                .unwrap()
                .fetch_variable("foo"),
            Some(Value::String("new value".to_string())),
        );
    }
}
