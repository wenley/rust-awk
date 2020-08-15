use regex::Regex;

pub enum Literal {
    StringLiteral(String),
    IntegerLiteral(u64),
    FloatLiteral(f64),
}

pub enum Expression {
    Literal(Literal)
}

#[derive(PartialEq,Debug)]
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
            Expression::Literal(literal) => {
                match literal {
                    Literal::StringLiteral(string) => { Value::String(string.clone()) }
                    Literal::IntegerLiteral(int) => { Value::Numeric(NumericValue::Integer(*int)) }
                    Literal::FloatLiteral(float) => { Value::Numeric(NumericValue::Float(*float)) }

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
        assert_eq!(Expression::Literal(Literal::StringLiteral("hello".to_string())).evaluate(), Value::String("hello".to_string()));
        assert_eq!(Expression::Literal(Literal::IntegerLiteral(0)).evaluate(), Value::Numeric(NumericValue::Integer(0)));
    }
}
