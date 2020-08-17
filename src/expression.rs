use regex::Regex;

use super::basic_types::Context;
use super::basic_types::NumericValue;
use super::basic_types::Value;

#[derive(Debug)]
pub enum Expression {
    StringLiteral(String),
    NumericLiteral(NumericValue),
    AddBinary {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Regex(Regex),
    Variable(String),
    FieldReference(Box<Expression>),
}

impl Expression {
    pub fn evaluate(&self, context: &Context) -> Value {
        match self {
            Expression::StringLiteral(string) => Value::String(string.clone()),
            Expression::NumericLiteral(numeric) => Value::Numeric(numeric.clone()),
            Expression::AddBinary { left, right } => {
                match (left.evaluate(context), right.evaluate(context)) {
                    (
                        Value::Numeric(NumericValue::Integer(x)),
                        Value::Numeric(NumericValue::Integer(y)),
                    ) => Value::Numeric(NumericValue::Integer(x + y)),
                    _ => panic!("Unsupported addition values {:?} and {:?}", left, right,),
                }
            }
            Expression::Regex(_) => {
                // Regex expressions shouldn't be evaluated as a standalone value, but should be
                // evaluated as part of explicit pattern matching operators. The one exception is
                // when a Regex is the pattern for an action, which is handled separately.
                //
                // When a Regex is evaluated on its own, it becomes an empty numeric string and
                // will be interpreted as such
                Value::Uninitialized
            }
            Expression::Variable(variable_name) => context.fetch_variable(variable_name),
            Expression::FieldReference(expression) => {
                let value = expression.evaluate(context).coerce_to_numeric();
                let index = match value {
                    NumericValue::Integer(i) => { i }
                    NumericValue::Float(f) => { f.floor() as i64 }
                };
                // TODO: Fetch this from Context
                Value::String("".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_can_evaluate() {
        let context = Context::empty();
        let string = Expression::StringLiteral("hello".to_string());
        assert_eq!(
            string.evaluate(&context),
            Value::String("hello".to_string())
        );
        let numeric = Expression::NumericLiteral(NumericValue::Integer(0));
        assert_eq!(
            numeric.evaluate(&context),
            Value::Numeric(NumericValue::Integer(0))
        );
    }

    #[test]
    fn variables_can_evaluate() {
        let mut context = Context::empty();
        let value = Value::Numeric(NumericValue::Integer(1));
        context.assign_variable("foo", value.clone());

        assert_eq!(
            Expression::Variable("foo".to_string()).evaluate(&context),
            value,
        );
    }

    #[test]
    fn binary_expressions_can_evaluate() {
        let context = Context::empty();
        assert_eq!(
            Expression::AddBinary {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(2))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Integer(3))),
            }
            .evaluate(&context),
            Value::Numeric(NumericValue::Integer(5)),
        );
    }
}
