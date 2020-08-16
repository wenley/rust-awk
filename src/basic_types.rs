use std::collections::HashMap;

pub enum Field {
    WholeLine,
    Indexed(usize),
}

pub struct Record<'a> {
    pub full_line: &'a str,
    pub fields: &'a Vec<&'a str>,
}

#[derive(PartialEq,Debug,Copy,Clone)]
pub enum NumericValue {
    Integer(u64),
    Float(f64),
}

#[derive(PartialEq,Debug)]
pub enum Value {
    String(String),
    Numeric(NumericValue),
    Uninitialized,
}

impl Value {
    pub fn coerce_to_string(&self) -> String {
        match self {
            Value::String(string) => { string.clone() }
            Value::Numeric(NumericValue::Integer(i)) => { i.to_string() }
            Value::Numeric(NumericValue::Float(f)) => { f.to_string() }
            Value::Uninitialized => { "".to_string() }
        }
    }

    pub fn coerce_to_numeric(&self) -> NumericValue {
        match self {
            Value::Numeric(n) => { *n }
            Value::String(string) => { panic!("Haven't implemented string to integer coercion") }
            Value::Uninitialized => { NumericValue::Integer(0) }
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::String(string) => { Value::String(string.clone()) }
            Value::Numeric(val) => { Value::Numeric(*val) }
            Value::Uninitialized => { Value::Uninitialized }
        }
    }
}

static UNINITIALIZED_VALUE: Value = Value::Uninitialized;

pub struct Context {
    variables: HashMap<String, Value>,
}

impl Context {
    pub fn empty() -> Context {
        Context {
            variables: HashMap::new(),
        }
    }

    pub fn fetch_variable(&self, variable_name: &str) -> Value {
        self.variables
            .get(variable_name)
            .map(|val| val.clone())
            .unwrap_or(UNINITIALIZED_VALUE.clone())
    }

    pub fn assign_variable(&mut self, variable_name: &str, value: Value) {
        self.variables.insert(
            variable_name.to_string(),
            value,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_values_coerce_to_strings() {
        assert_eq!(Value::String("hello".to_string()).coerce_to_string(), "hello");
        assert_eq!(Value::Numeric(NumericValue::Integer(123)).coerce_to_string(), "123");
        assert_eq!(Value::Numeric(NumericValue::Float(1.234)).coerce_to_string(), "1.234");
        assert_eq!(Value::Uninitialized.coerce_to_string(), "");
    }
}
