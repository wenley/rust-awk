use regex::Regex;

use super::basic_types::Context;
use super::basic_types::NumericValue;
use super::basic_types::Value;
use super::basic_types::Record;

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
    pub fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        match self {
            Expression::StringLiteral(string) => Value::String(string.clone()),
            Expression::NumericLiteral(numeric) => Value::Numeric(numeric.clone()),
            Expression::AddBinary { left, right } => {
                match (left.evaluate(context, record), right.evaluate(context, record)) {
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
                let value = expression.evaluate(context, record).coerce_to_numeric();
                let unsafe_index = match value {
                    NumericValue::Integer(i) => { i }
                    NumericValue::Float(f) => { f.floor() as i64 }
                };
                if unsafe_index < 0 {
                    panic!("Field indexes cannot be negative: {}", unsafe_index);
                }
                let index = unsafe_index as usize;

                record.fields
                    .get(index)
                    .map(|s| Value::String(s.to_string()))
                    .unwrap_or(Value::Uninitialized)
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
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };
        let string = Expression::StringLiteral("hello".to_string());
        assert_eq!(
            string.evaluate(&context, &record),
            Value::String("hello".to_string())
        );
        let numeric = Expression::NumericLiteral(NumericValue::Integer(0));
        assert_eq!(
            numeric.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0))
        );
    }

    #[test]
    fn variables_can_evaluate() {
        let mut context = Context::empty();
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };
        let value = Value::Numeric(NumericValue::Integer(1));
        context.assign_variable("foo", value.clone());

        assert_eq!(
            Expression::Variable("foo".to_string()).evaluate(&context, &record),
            value,
        );
    }

    #[test]
    fn binary_expressions_can_evaluate() {
        let context = Context::empty();
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };
        assert_eq!(
            Expression::AddBinary {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(2))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Integer(3))),
            }
            .evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(5)),
        );
    }
}
