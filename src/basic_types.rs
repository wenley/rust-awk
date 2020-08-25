use regex;
use std::collections::HashMap;

pub struct Record<'a> {
    pub full_line: &'a str,
    pub fields: &'a Vec<&'a str>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum NumericValue {
    Integer(i64),
    Float(f64),
}

#[derive(PartialEq, Debug)]
pub enum Value {
    String(String),
    Numeric(NumericValue),
    Uninitialized,
}

impl Value {
    pub fn coerce_to_string(&self) -> String {
        match self {
            Value::String(string) => string.clone(),
            Value::Numeric(NumericValue::Integer(i)) => i.to_string(),
            Value::Numeric(NumericValue::Float(f)) => f.to_string(),
            Value::Uninitialized => "".to_string(),
        }
    }

    pub fn coerce_to_numeric(&self) -> NumericValue {
        match self {
            Value::Numeric(n) => *n,
            Value::String(_) => panic!("Haven't implemented string to integer coercion"),
            Value::Uninitialized => NumericValue::Integer(0),
        }
    }

    pub fn coercion_to_boolean(&self) -> bool {
        match self {
            Value::String(s) => match s.as_str() {
                "" => false,
                _ => true,
            },
            Value::Numeric(n) => match n {
                NumericValue::Integer(0) | NumericValue::Float(0.0) => false,
                _ => true,
            },
            Value::Uninitialized => false,
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::String(string) => Value::String(string.clone()),
            Value::Numeric(val) => Value::Numeric(*val),
            Value::Uninitialized => Value::Uninitialized,
        }
    }
}

static UNINITIALIZED_VALUE: Value = Value::Uninitialized;

enum FieldSeparator {
    Character(char),
    Regex(regex::Regex),
}

pub struct Context {
    field_separator: FieldSeparator,
    variables: HashMap<String, Value>,
}

impl Context {
    pub fn empty() -> Context {
        Context {
            field_separator: FieldSeparator::Character(' '),
            variables: HashMap::new(),
        }
    }

    pub fn fetch_variable(&self, variable_name: &str) -> Value {
        self.variables
            .get(variable_name)
            .map(|val| val.clone())
            .unwrap_or(UNINITIALIZED_VALUE.clone())
    }

    pub fn set_field_separator(&mut self, new_separator: &str) {
        if new_separator.len() == 1 {
            self.field_separator = FieldSeparator::Character(new_separator.chars().next().unwrap())
        } else {
            self.field_separator = FieldSeparator::Regex(regex::Regex::new(new_separator).unwrap())
        }
    }

    pub fn assign_variable(&mut self, variable_name: &str, value: Value) {
        self.variables.insert(variable_name.to_string(), value);
    }

    pub fn split<'a>(&self, line: &'a str) -> Vec<&'a str> {
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
}
