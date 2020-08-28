use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::{many0, separated_list},
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};

use crate::{
    basic_types::{Context, Record},
    expression::{parse_expression, Expression},
};

static EMPTY_STRING: &str = "";

#[derive(PartialEq, Debug)]
pub(crate) enum Statement {
    IfElse {
        condition: Expression,
        if_branch: Box<Statement>,   // TODO: This should be an Action
        else_branch: Box<Statement>, // TODO: This should be an Action
    },
    Print(Vec<Expression>),
    Assign {
        variable_name: String,
        value: Expression,
    },
    While {
        condition: Expression,
        body: Box<Statement>, // TODO: This should be an Action
    },
}

impl Statement {
    pub fn evaluate<'a>(&self, context: &mut Context, record: &'a Record) -> String {
        match self {
            Statement::Print(expressions) => expressions
                .iter()
                .map(|e| e.evaluate(context, record).coerce_to_string())
                .collect::<Vec<String>>()
                .join(" "),
            Statement::IfElse {
                condition,
                if_branch,
                else_branch,
            } => {
                let result = condition.evaluate(context, record).coercion_to_boolean();
                if result {
                    if_branch.evaluate(context, record)
                } else {
                    else_branch.evaluate(context, record)
                }
            }
            Statement::Assign {
                variable_name,
                value,
            } => {
                let value = value.evaluate(context, record);
                context.assign_variable(&variable_name, value);
                EMPTY_STRING.to_string()
            }
            Statement::While { condition, body } => {
                let mut value = condition.evaluate(context, record);
                loop {
                    if value.coercion_to_boolean() {
                        let _maybe_string_to_print = body.evaluate(context, record);
                        value = condition.evaluate(context, record);
                    } else {
                        break;
                    }
                }
                // TODO: This should return a vec![] of strings, to be combined
                // with at the Action level
                EMPTY_STRING.to_string()
            }
        }
    }
}

pub(crate) fn parse_statements(input: &str) -> IResult<&str, Vec<Statement>> {
    let parse_single_statement = terminated(
        parse_simple_statement,
        tuple((multispace0, one_of(";"), multispace0)),
    );
    many0(parse_single_statement)(input)
}

fn parse_simple_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        parse_print_statement,
        parse_if_else_statement,
        map(parse_expression, move |expr| Statement::Print(vec![expr])),
    ))(input)
}

fn parse_print_statement(input: &str) -> IResult<&str, Statement> {
    let parse_separator = delimited(multispace0, one_of(","), multispace0);
    let parse_expression_list = separated_list(parse_separator, parse_expression);
    map(
        delimited(tag("print("), parse_expression_list, one_of(")")),
        move |exprs| Statement::Print(exprs),
    )(input)
}

fn parse_if_else_statement(input: &str) -> IResult<&str, Statement> {
    let parse_if_open = tuple((tag("if"), multispace0, one_of("("), multispace0));
    let parse_if_close = tuple((
        multispace0,
        one_of(")"),
        multispace0,
        one_of("{"),
        multispace0,
    ));
    let parse_else_open = tuple((
        multispace0,
        one_of("}"),
        multispace0,
        tag("else"),
        multispace0,
        one_of("{"),
        multispace0,
    ));
    let parse_else_close = pair(multispace0, one_of("}"));
    let parse_if = delimited(parse_if_open, parse_expression, parse_if_close);

    map(
        tuple((
            parse_if,
            parse_simple_statement,
            parse_else_open,
            parse_simple_statement,
            parse_else_close,
        )),
        move |(condition, if_branch, _, else_branch, _)| Statement::IfElse {
            condition: condition,
            if_branch: Box::new(if_branch),
            else_branch: Box::new(else_branch),
        },
    )(input)
}

fn parse_while_statement(input: &str) -> IResult<&str, Statement> {
    let parse_while_condition_open = tuple((tag("while"), multispace0, one_of("("), multispace0));
    let parse_while_condition_close = tuple((
        multispace0,
        one_of(")"),
        multispace0,
        one_of("{"),
        multispace0,
    ));
    let parse_while_condition = delimited(
        parse_while_condition_open,
        parse_expression,
        parse_while_condition_close,
    );

    let parse_while_close = pair(multispace0, one_of("}"));

    map(
        tuple((
            parse_while_condition,
            parse_simple_statement,
            parse_while_close,
        )),
        move |(condition, body, _)| Statement::While {
            condition: condition,
            body: Box::new(body),
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::NumericValue;

    #[test]
    fn test_parse_statements() {
        let result = parse_print_statement(r#"print("hello")"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            Statement::Print(vec![Expression::StringLiteral("hello".to_string())])
        );

        let result = parse_statements(
            r#"print(1);
            print(2.0   ,    "extra arg");
            print("hello");
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            vec![
                Statement::Print(vec![Expression::NumericLiteral(NumericValue::Integer(1))]),
                Statement::Print(vec![
                    Expression::NumericLiteral(NumericValue::Float(2.0)),
                    Expression::StringLiteral("extra arg".to_string()),
                ]),
                Statement::Print(vec![Expression::StringLiteral("hello".to_string())]),
            ],
        );
    }

    #[test]
    fn test_parse_if_else_statement() {
        // This test is "not correct" because the if and else branches are currently
        // parsed as single statements, rather than as a full Action
        let result = parse_simple_statement(
            r#"if (1) {
            print("hello")
        } else { "noop" }"#,
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            Statement::IfElse {
                condition: Expression::NumericLiteral(NumericValue::Integer(1)),
                if_branch: Box::new(Statement::Print(vec![Expression::StringLiteral(
                    "hello".to_string()
                )])),
                else_branch: Box::new(Statement::Print(vec![Expression::StringLiteral(
                    "noop".to_string()
                )])),
            },
        );
    }

    #[test]
    fn test_parse_while_statement() {
        // This test is "not correct" because the body is currently
        // parsed as a single statement, rather than as a full Action
        let result = parse_while_statement(
            r#"while (0) {
                print("hello")
            }"#,
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            Statement::While {
                condition: Expression::NumericLiteral(NumericValue::Integer(0)),
                body: Box::new(Statement::Print(vec![Expression::StringLiteral(
                    "hello".to_string()
                )])),
            },
        );
    }
}
