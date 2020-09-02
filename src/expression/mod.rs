use regex::Regex;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, multispace0, none_of, one_of},
    combinator::{map, map_res},
    multi::{many0, many1},
    re_find,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

use crate::{
    basic_types::{Context, Record},
    value::{parse_float_literal, parse_integer_literal, NumericValue, Value},
};

#[derive(Debug)]
pub(crate) enum Expression {
    StringLiteral(String),
    NumericLiteral(NumericValue),
    AddBinary {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Regex(Regex),
    Variable(String),
    FieldReference(Box<Expression>),
    RegexMatch {
        left: Box<Expression>,
        right: Box<Expression>,
        negated: bool,
    },
}

impl Expression {
    pub(crate) fn evaluate<'a>(&self, context: &Context, record: &'a Record) -> Value {
        match self {
            Expression::StringLiteral(string) => Value::String(string.clone()),
            Expression::NumericLiteral(numeric) => Value::Numeric(numeric.clone()),
            Expression::AddBinary { left, right } => {
                match (
                    left.evaluate(context, record),
                    right.evaluate(context, record),
                ) {
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
                    NumericValue::Integer(i) => i,
                    NumericValue::Float(f) => f.floor() as i64,
                };
                match unsafe_index {
                    i if i < 0 => panic!("Field indexes cannot be negative: {}", unsafe_index),
                    i if i == 0 => Value::String(record.full_line.to_string()),
                    i => record
                        .fields
                        .get((i - 1) as usize)
                        .map(|s| Value::String(s.to_string()))
                        .unwrap_or(Value::Uninitialized),
                }
            }
            Expression::RegexMatch {
                left,
                right,
                negated,
            } => {
                let left_value = left.evaluate(context, record).coerce_to_string();

                let matches = match &**right {
                    Expression::Regex(regex) => regex.is_match(&left_value),
                    _ => {
                        let value = right.evaluate(context, record).coerce_to_string();
                        Regex::new(&value).unwrap().is_match(&left_value)
                    }
                };

                let int_value = if matches ^ negated { 1 } else { 0 };

                Value::Numeric(NumericValue::Integer(int_value))
            }
        }
    }
}

impl PartialEq for Expression {
    fn eq(&self, other: &Expression) -> bool {
        match (self, other) {
            (Expression::StringLiteral(s1), Expression::StringLiteral(s2)) => s1 == s2,
            (Expression::NumericLiteral(n1), Expression::NumericLiteral(n2)) => n1 == n2,
            (Expression::Regex(r1), Expression::Regex(r2)) => r1.as_str() == r2.as_str(),
            (Expression::Variable(s1), Expression::Variable(s2)) => s1 == s2,
            (Expression::FieldReference(s1), Expression::FieldReference(s2)) => s1 == s2,
            (
                Expression::AddBinary {
                    left: l1,
                    right: r1,
                },
                Expression::AddBinary {
                    left: l2,
                    right: r2,
                },
            ) => l1 == l2 && r1 == r2,
            (
                Expression::RegexMatch {
                    left: l1,
                    right: r1,
                    negated: n1,
                },
                Expression::RegexMatch {
                    left: l2,
                    right: r2,
                    negated: n2,
                },
            ) => l1 == l2 && r1 == r2 && n1 == n2,
            _ => false,
        }
    }
}

/// Tiers of parsing
///
/// The top-level parser is responsible for the loosest-binding / lowest-precedence
/// operators. As we descend the levels, we encounter tighter-binding operators
/// until we reach literals and the parenthesized expressions.

pub(crate) fn parse_expression(input: &str) -> IResult<&str, Expression> {
    alt((parse_regex_match, parse_addition))(input)
}

// Regex matching does not associate
fn parse_regex_match(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            parse_addition,
            delimited(multispace0, alt((tag("~"), tag("!~"))), multispace0),
            parse_addition,
        )),
        |(left, operator, right)| {
            let negated = match operator.len() {
                1 => false,
                2 => true,
                _ => panic!("Unexpected regex operator length: {}", operator),
            };
            Expression::RegexMatch {
                left: Box::new(left),
                right: Box::new(right),
                negated,
            }
        },
    )(input)
}

fn parse_addition(input: &str) -> IResult<&str, Expression> {
    let parse_added_expr = map(
        pair(
            delimited(multispace0, one_of("+"), multispace0),
            parse_primary,
        ),
        |(_, rhs)| rhs,
    );
    map(
        pair(parse_primary, many0(parse_added_expr)),
        move |(first, mut rest)| {
            rest.drain(0..)
                .fold(first, |inner, next| Expression::AddBinary {
                    left: Box::new(inner),
                    right: Box::new(next),
                })
        },
    )(input)
}

fn parse_primary(input: &str) -> IResult<&str, Expression> {
    alt((
        parse_literal,
        parse_variable,
        parse_parens,
        parse_field_reference,
    ))(input)
}

fn parse_parens(input: &str) -> IResult<&str, Expression> {
    delimited(one_of("("), parse_expression, one_of(")"))(input)
}

fn parse_variable(input: &str) -> IResult<&str, Expression> {
    map(alpha1, |name: &str| Expression::Variable(name.to_string()))(input)
}

fn parse_literal(input: &str) -> IResult<&str, Expression> {
    alt((
        parse_string_literal,
        parse_regex_literal,
        parse_number_literal,
    ))(input)
}

fn parse_number_literal(input: &str) -> IResult<&str, Expression> {
    map(
        alt((parse_float_literal, parse_integer_literal)),
        |number| Expression::NumericLiteral(number),
    )(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Expression> {
    map(
        delimited(one_of("\""), parse_string_contents, one_of("\"")),
        |contents: &str| Expression::StringLiteral(contents.to_string()),
    )(input)
}

fn parse_string_contents(input: &str) -> IResult<&str, &str> {
    //
    // Allow strings to contain sequences of:
    // 1. Non-slash non-double-quote characters
    // 2. Slash followed anything (including a double-quote)
    //
    re_find!(input, r#"^([^\\"]|\\.)*"#)
}

fn parse_regex_literal(input: &str) -> IResult<&str, Expression> {
    let parser = tuple((one_of("/"), many1(none_of("/")), one_of("/")));
    map_res(parser, |(_, vec, _)| {
        regex::Regex::new(&vec.iter().collect::<String>()).map(|regex| Expression::Regex(regex))
    })(input)
}

fn parse_field_reference(input: &str) -> IResult<&str, Expression> {
    map(preceded(one_of("$"), parse_expression), |expr| {
        Expression::FieldReference(Box::new(expr))
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_context_and_record() -> (Context, Record<'static>) {
        (
            Context::empty(),
            Record {
                full_line: "",
                fields: vec![],
            },
        )
    }

    #[test]
    fn literals_can_evaluate() {
        let (context, record) = empty_context_and_record();
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
        let (mut context, record) = empty_context_and_record();
        let value = Value::Numeric(NumericValue::Integer(1));
        context.assign_variable("foo", value.clone());

        assert_eq!(
            Expression::Variable("foo".to_string()).evaluate(&context, &record),
            value,
        );
    }

    #[test]
    fn binary_expressions_can_evaluate() {
        let (context, record) = empty_context_and_record();
        assert_eq!(
            Expression::AddBinary {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(2))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Integer(3))),
            }
            .evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(5)),
        );
    }

    #[test]
    fn field_reference_can_evaluate() {
        let (context, mut record) = empty_context_and_record();
        record.fields = vec!["first", "second"];

        assert_eq!(
            Expression::FieldReference(Box::new(Expression::NumericLiteral(
                NumericValue::Integer(1)
            )))
            .evaluate(&context, &record),
            Value::String("first".to_string()),
        );
    }

    #[test]
    fn parse_expressions() {
        let result = parse_expression("1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1,
            Expression::NumericLiteral(NumericValue::Integer(1))
        );

        let result = parse_string_literal(r#""hello""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            Expression::StringLiteral("hello".to_string()),
        );

        let result = parse_expression(r#""hello world""#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            Expression::StringLiteral("hello world".to_string()),
        );

        let result = parse_expression("(1)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1,
            Expression::NumericLiteral(NumericValue::Integer(1))
        );

        let result = parse_expression("(1) + (2.5)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1,
            Expression::AddBinary {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(1))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Float(2.5))),
            }
        );
        let result = parse_expression("(1) + (2.5)");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1,
            Expression::AddBinary {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(1))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Float(2.5))),
            }
        );
    }

    #[test]
    fn test_parse_field_reference() {
        let result = parse_expression("$1");
        assert_eq!(result.is_ok(), true);
        assert_eq!(
            result.unwrap().1,
            Expression::FieldReference(Box::new(Expression::NumericLiteral(
                NumericValue::Integer(1)
            )),),
        );
    }

    #[test]
    fn test_regex_match() {
        let (context, record) = empty_context_and_record();

        let result = parse_expression("1 ~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression,
            Expression::RegexMatch {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(1))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Integer(2))),
                negated: false,
            },
        );
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(0)),
        );

        // Cannot consume the full expression
        let result = parse_expression("1 ~ 2 ~ 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, " ~ 3");

        let result = parse_expression("1 + 2 ~ 3");
        assert!(result.is_ok());
        let (remainder, expression) = result.unwrap();
        assert_eq!(remainder, "");
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );

        let result = parse_expression("1 !~ 2");
        assert!(result.is_ok());
        let expression = result.unwrap().1;
        assert_eq!(
            expression,
            Expression::RegexMatch {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(1))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Integer(2))),
                negated: true,
            },
        );
        assert_eq!(
            expression.evaluate(&context, &record),
            Value::Numeric(NumericValue::Integer(1)),
        );
    }
}
