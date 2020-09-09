#[derive(PartialEq, Debug, Copy, Clone)]
pub(crate) enum NumericValue {
    Integer(i64),
    Float(f64),
}

#[derive(PartialEq, Debug)]
pub(crate) enum Value {
    String(String),
    Numeric(NumericValue),
    Uninitialized,
}

impl Value {
    pub(crate) fn coerce_to_string(&self) -> String {
        match self {
            Value::String(string) => string.clone(),
            Value::Numeric(NumericValue::Integer(i)) => i.to_string(),
            Value::Numeric(NumericValue::Float(f)) => f.to_string(),
            Value::Uninitialized => "".to_string(),
        }
    }

    pub(crate) fn coerce_to_numeric(&self) -> NumericValue {
        match self {
            Value::Numeric(n) => *n,
            Value::String(_) => panic!("Haven't implemented string to integer coercion"),
            Value::Uninitialized => NumericValue::Integer(0),
        }
    }

    pub(crate) fn coercion_to_boolean(&self) -> bool {
        match self {
            Value::String(s) => match s.as_str() {
                "" => false,
                _ => true,
            },
            Value::Numeric(n) => match n {
                NumericValue::Integer(0) => false,
                NumericValue::Float(f) => *f != 0.0,
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_string_to_numeric() {
        assert_eq!(
            Value::String("123".to_string()).coerce_to_numeric(),
            NumericValue::Integer(123),
        );
    }
}
