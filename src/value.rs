use nom::{character::complete::multispace0, re_find, sequence::preceded, IResult};
use regex::Regex;

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
            Value::String(s) => match preceded(multispace0, parse_numeric)(s) {
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

pub(crate) fn parse_numeric(input: &str) -> IResult<&str, NumericValue> {
    let (input, matched) = re_find!(input, r"^[-+]?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?")?;

    // Once we know we have a good decimal, look for the different parts
    // Use `+?` to ensure blanks are captured as None for easier matching
    let parts: Regex = Regex::new(r"^[-+]?(?P<digits>[0-9]+)?(?P<dot>\.)?(?P<decimals>[0-9]+)?([eE](?P<exponent>[-+]?[0-9]+))?").unwrap();
    let captures = parts.captures(matched).unwrap();

    match (
        captures.name("digits"),
        captures.name("dot"),
        captures.name("decimals"),
        captures.name("exponent"),
    ) {
        (Some(_), _, None, None) | (_, None, Some(_), None) => {
            IResult::Ok((input, parse_as_int(matched)))
        }
        (Some(digits_string), _, None, Some(exponent_string)) => {
            let mut exponent = exponent_string.as_str().parse::<i64>().unwrap();
            let mut digits = digits_string.as_str().parse::<i64>().unwrap();

            // Simplification by converting trailing zeroes into the exponent
            while digits % 10 == 0 {
                digits = digits / 10;
                exponent = exponent + 1;
            }

            // If after simplification, the number has no decimal digits, then it is considered an
            // integer
            if exponent >= 0 {
                IResult::Ok((
                    input,
                    NumericValue::Integer(matched.parse::<f64>().unwrap() as i64),
                ))
            } else {
                IResult::Ok((input, parse_as_float(matched)))
            }
        }
        (_, Some(_), Some(decimal_string), Some(exponent_string)) => {
            let exponent = exponent_string.as_str().parse::<i64>().unwrap();
            let num_decimals = decimal_string
                .as_str()
                .trim_end_matches("0")
                .chars()
                .count() as i64;

            // If after simplification, the number has no decimal digits, then it is considered an
            // integer
            if exponent >= num_decimals {
                IResult::Ok((
                    input,
                    NumericValue::Integer(matched.parse::<f64>().unwrap() as i64),
                ))
            } else {
                IResult::Ok((input, parse_as_float(matched)))
            }
        }
        (_, _, _, _) => IResult::Ok((input, parse_as_float(matched))),
    }
}

fn parse_as_float(s: &str) -> NumericValue {
    NumericValue::Float(s.parse::<f64>().unwrap())
}

fn parse_as_int(s: &str) -> NumericValue {
    NumericValue::Integer(s.parse::<f64>().unwrap() as i64)
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
        assert_eq!(
            Value::String("-12e3".to_string()).coerce_to_numeric(),
            NumericValue::Integer(-12000),
        );
        assert_eq!(
            Value::String("-12e-3".to_string()).coerce_to_numeric(),
            NumericValue::Float(-12e-3),
        );
        assert_eq!(
            Value::String("-.12e2".to_string()).coerce_to_numeric(),
            NumericValue::Integer(-12),
        );
        assert_eq!(
            Value::String("-.12e1".to_string()).coerce_to_numeric(),
            NumericValue::Float(-1.2),
        );
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
        assert_eq!(parse_numeric("123").unwrap().1, NumericValue::Integer(123));
        assert_eq!(
            parse_numeric("123000").unwrap().1,
            NumericValue::Integer(123000)
        );
        assert_eq!(
            parse_numeric("-123").unwrap().1,
            NumericValue::Integer(-123)
        );
        assert_eq!(parse_numeric("(123").is_err(), true);
        // Would like this test to pass, but the distinction is implemented
        // by the sequencing of the parsers of parse_number_literal
        // assert_eq!(parse_numeric("123.45").is_err(), true);
        assert_eq!(parse_numeric(".").is_err(), true);

        // Floats
        assert_eq!(
            parse_numeric("123.45"),
            IResult::Ok(("", NumericValue::Float(123.45)))
        );
        assert_eq!(
            parse_numeric("123.45e-5"),
            IResult::Ok(("", NumericValue::Float(123.45e-5)))
        );
        assert_eq!(
            parse_numeric("123.45E5"),
            IResult::Ok(("", NumericValue::Integer(12345000)))
        );
        assert_eq!(
            parse_numeric(".45"),
            IResult::Ok(("", NumericValue::Float(0.45)))
        );
        assert_eq!(
            parse_numeric("-123.45"),
            IResult::Ok(("", NumericValue::Float(-123.45)))
        );
        assert_eq!(parse_numeric("a").is_err(), true);
        assert_eq!(parse_numeric(".").is_err(), true);
        assert_eq!(parse_numeric("+e").is_err(), true);
    }
}
