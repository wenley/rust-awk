use std::collections::HashMap;

use crate::value::Value;

pub(crate) struct StackFrame {
    variables: HashMap<String, Value>,
}

impl StackFrame {
    pub(crate) fn empty() -> StackFrame {
        StackFrame {
            variables: HashMap::new(),
        }
    }

    pub(super) fn fetch_variable(&self, variable_name: &str) -> Option<Value> {
        self.variables.get(variable_name).map(|val| val.clone())
    }

    pub(crate) fn assign_variable(&mut self, variable_name: &str, value: Value) {
        self.variables.insert(variable_name.to_string(), value);
    }
}
