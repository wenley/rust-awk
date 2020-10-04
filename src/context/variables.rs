use regex;

use crate::context::{stack_frame::StackFrame, Record, VariableStore};
use crate::value::{Value, UNINITIALIZED_VALUE};

pub(super) enum FieldSeparator {
    Character(char),
    Regex(regex::Regex),
}

pub(crate) struct Variables {
    pub(super) field_separator: FieldSeparator,
    pub(super) global_variables: StackFrame,
    pub(super) function_variables: Vec<StackFrame>,
}

impl Variables {
    pub(crate) fn empty() -> Variables {
        Variables {
            field_separator: FieldSeparator::Character(' '),
            global_variables: StackFrame::empty(),
            function_variables: vec![],
        }
    }

    pub(super) fn set_field_separator(&mut self, new_separator: &str) {
        if new_separator.len() == 1 {
            self.field_separator = FieldSeparator::Character(new_separator.chars().next().unwrap())
        } else {
            self.field_separator = FieldSeparator::Regex(regex::Regex::new(new_separator).unwrap())
        }
    }

    pub(super) fn record_for_line<'a>(&self, line: &'a str) -> Record<'a> {
        let fields = match &self.field_separator {
            FieldSeparator::Character(' ') => line.split_whitespace().collect(),
            FieldSeparator::Character(c1) => line.split(|c2| *c1 == c2).collect(),
            FieldSeparator::Regex(re) => re.split(line).collect(),
        };
        Record {
            full_line: line,
            fields: fields,
        }
    }
}

impl VariableStore for Variables {
    fn fetch_variable(&self, variable_name: &str) -> Value {
        let last_frame = self.function_variables.last();

        last_frame
            .and_then(|frame| frame.fetch_variable(variable_name))
            .or_else(|| self.global_variables.fetch_variable(variable_name))
            .unwrap_or_else(|| UNINITIALIZED_VALUE.clone())
    }

    fn assign_variable(&mut self, variable_name: &str, value: Value) {
        if let Some(frame) = self.function_variables.last_mut() {
            if let Some(_) = frame.fetch_variable(variable_name) {
                frame.assign_variable(variable_name, value);
                return;
            }
        }

        if variable_name == "FS" {
            self.set_field_separator(&value.coerce_to_string());
        }
        self.global_variables.assign_variable(variable_name, value);
    }
}
