
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
}

pub struct Context {
}

