use nom::{branch::alt, character::complete::multispace0, re_find, sequence::preceded, IResult};

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
            Value::String(s) => match preceded(
                multispace0,
                alt((parse_float_literal, parse_integer_literal)),
            )(s)
            {
                Ok((_, n)) => n,
                Err(_) => NumericValue::Float(0.0),
            },
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

pub(crate) fn parse_float_literal(input: &str) -> IResult<&str, NumericValue> {
    // Omit ? on the . to intentionally _not_ match on integers
    let (input, matched) = re_find!(input, r"^[-+]?[0-9]*\.[0-9]+([eE][-+]?[0-9]+)?")?;
    let number = matched.parse::<f64>().unwrap();

    IResult::Ok((input, NumericValue::Float(number)))
}

pub(crate) fn parse_integer_literal(input: &str) -> IResult<&str, NumericValue> {
    let (input, matched) = re_find!(input, r"^[-+]?[0-9]+")?;
    let number = matched.parse::<i64>().unwrap();

    IResult::Ok((input, NumericValue::Integer(number)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_numeric() {
        assert_eq!(
            Value::String("123".to_string()).coerce_to_numeric(),
            NumericValue::Integer(123),
        );
        assert_eq!(
            Value::String("1.23".to_string()).coerce_to_numeric(),
            NumericValue::Float(1.23),
        );
        // TODO: Make these tests work
        // assert_eq!(
        //     Value::String("-12e3".to_string()).coerce_to_numeric(),
        //     NumericValue::Integer(-12000),
        // );
        // assert_eq!(
        //     Value::String("-12e-3".to_string()).coerce_to_numeric(),
        //     NumericValue::Float(-12e-3),
        // );
        assert_eq!(
            Value::String("       123".to_string()).coerce_to_numeric(),
            NumericValue::Integer(123),
        );
        assert_eq!(
            Value::String("123abc".to_string()).coerce_to_numeric(),
            NumericValue::Integer(123),
        );
    }

    #[test]
    fn parse_number_literals() {
        // Integers
        assert_eq!(
            parse_integer_literal("123").unwrap().1,
            NumericValue::Integer(123)
        );
        assert_eq!(
            parse_integer_literal("123000").unwrap().1,
            NumericValue::Integer(123000)
        );
        assert_eq!(
            parse_integer_literal("-123").unwrap().1,
            NumericValue::Integer(-123)
        );
        assert_eq!(parse_integer_literal("(123").is_err(), true);
        // Would like this test to pass, but the distinction is implemented
        // by the sequencing of the parsers of parse_number_literal
        // assert_eq!(parse_integer_literal("123.45").is_err(), true);
        assert_eq!(parse_integer_literal(".").is_err(), true);

        // Floats
        assert_eq!(
            parse_float_literal("123.45"),
            IResult::Ok(("", NumericValue::Float(123.45)))
        );
        assert_eq!(
            parse_float_literal("123.45e-5"),
            IResult::Ok(("", NumericValue::Float(123.45e-5)))
        );
        assert_eq!(
            parse_float_literal("123.45E5"),
            IResult::Ok(("", NumericValue::Float(123.45e5)))
        );
        assert_eq!(
            parse_float_literal(".45"),
            IResult::Ok(("", NumericValue::Float(0.45)))
        );
        assert_eq!(
            parse_float_literal("-123.45"),
            IResult::Ok(("", NumericValue::Float(-123.45)))
        );
        assert_eq!(parse_float_literal("a").is_err(), true);
        assert_eq!(parse_float_literal(".").is_err(), true);
        assert_eq!(parse_float_literal("+e").is_err(), true);
    }
}
