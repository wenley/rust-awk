use regex::Regex;

#[derive(Debug)]
pub enum Expression {
    StringLiteral(String),
    NumericLiteral(NumericValue),
    AddBinary { left: Box<Expression>, right: Box<Expression> },
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

impl Expression {
    pub fn evaluate(&self) -> Value {
        match self {
            Expression::StringLiteral(string) => { Value::String(string.clone()) }
            Expression::NumericLiteral(numeric) => { Value::Numeric(numeric.clone()) }
            Expression::AddBinary { left: left, right: right } => {
                match (left.evaluate(), right.evaluate()) {
                    (Value::Numeric(NumericValue::Integer(x)), Value::Numeric(NumericValue::Integer(y))) => { Value::Numeric(NumericValue::Integer(x + y)) }
                    _ => { panic!("Unsupported addition values {:?} and {:?}", left, right) }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_can_evaluate() {
        assert_eq!(Expression::StringLiteral("hello".to_string()).evaluate(), Value::String("hello".to_string()));
        assert_eq!(Expression::NumericLiteral(NumericValue::Integer(0)).evaluate(), Value::Numeric(NumericValue::Integer(0)));
    }

    #[test]
    fn variables_can_evaluate() {
    }

    #[test]
    fn binary_expressions_can_evaluate() {
        assert_eq!(
            Expression::AddBinary {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(2))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Integer(3))),
            }.evaluate(),
            Value::Numeric(NumericValue::Integer(5)),
        );
    }
}
