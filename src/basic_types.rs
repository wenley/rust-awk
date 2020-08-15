use std::collections::HashMap;

pub enum Field {
    WholeLine,
    Indexed(usize),
}

pub struct Record<'a> {
    pub full_line: &'a str,
    pub fields: &'a Vec<&'a str>,
}

#[derive(PartialEq,Debug,Clone)]
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

    pub fn fetch_variable<'a>(&'a self, variable_name: &str) -> &'a Value {
        self.variables.get(variable_name).unwrap_or(&UNINITIALIZED_VALUE)
    }
}
