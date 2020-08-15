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

    pub fn fetch_variable<'a>(&'a self, variable_name: &str) -> Value {
        self.variables
            .get(variable_name)
            .map(|val| val.clone())
            .unwrap_or(UNINITIALIZED_VALUE.clone())
    }
}
