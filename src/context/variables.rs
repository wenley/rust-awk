use regex;

use crate::context::{Record, StackFrame};

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
