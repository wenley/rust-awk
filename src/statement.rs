use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::{many0, separated_list},
    sequence::{delimited, terminated, tuple},
    IResult,
};

use crate::{
    basic_types::{Context, Record},
    expression::{parse_expression, Expression},
};

#[derive(PartialEq, Debug)]
pub(crate) enum Statement {
    IfElse {
        condition: Expression,
        if_branch: Action,
        else_branch: Action,
    },
    Print(Vec<Expression>),
    Assign {
        variable_name: String,
        value: Expression,
    },
    While {
        condition: Expression,
        body: Action,
    },
}

impl Statement {
    pub fn evaluate<'a>(&self, context: &mut Context, record: &'a Record) -> Vec<String> {
        match self {
            Statement::Print(expressions) => {
                let output_line = expressions
                    .iter()
                    .map(|e| e.evaluate(context, record).coerce_to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                vec![output_line]
            }
            Statement::IfElse {
                condition,
                if_branch,
                else_branch,
            } => {
                let result = condition.evaluate(context, record).coercion_to_boolean();
                if result {
                    if_branch.output_for_line(context, record)
                } else {
                    else_branch.output_for_line(context, record)
                }
            }
            Statement::Assign {
                variable_name,
                value,
            } => {
                let value = value.evaluate(context, record);
                context.assign_variable(&variable_name, value);
                vec![]
            }
            Statement::While { condition, body } => {
                let mut value = condition.evaluate(context, record);
                let mut output = vec![];
                loop {
                    if value.coercion_to_boolean() {
                        output.append(&mut body.output_for_line(context, record));
                        value = condition.evaluate(context, record);
                    } else {
                        break;
                    }
                }
                output
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Action {
    pub(crate) statements: Vec<Statement>,
}

impl Action {
    pub fn output_for_line<'a>(&self, context: &mut Context, record: &Record<'a>) -> Vec<String> {
        self.statements
            .iter()
            .flat_map(|statement| statement.evaluate(context, record))
            .collect()
    }
}

pub(crate) fn parse_action(input: &str) -> IResult<&str, Action> {
    map(
        delimited(
            tuple((one_of("{"), multispace0)),
            parse_statements,
            tuple((multispace0, one_of("}"))),
        ),
        move |statements| Action {
            statements: statements,
        },
    )(input)
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
    let parse_if_close = tuple((multispace0, one_of(")"), multispace0));
    let parse_else_open = tuple((multispace0, tag("else"), multispace0));
    let parse_if = delimited(parse_if_open, parse_expression, parse_if_close);

    map(
        tuple((parse_if, parse_action, parse_else_open, parse_action)),
        move |(condition, if_branch, _, else_branch)| Statement::IfElse {
            condition: condition,
            if_branch: if_branch,
            else_branch: else_branch,
        },
    )(input)
}

fn parse_while_statement(input: &str) -> IResult<&str, Statement> {
    let parse_while_condition_open = tuple((tag("while"), multispace0, one_of("("), multispace0));
    let parse_while_condition_close = tuple((multispace0, one_of(")"), multispace0));
    let parse_while_condition = delimited(
        parse_while_condition_open,
        parse_expression,
        parse_while_condition_close,
    );

    map(
        tuple((parse_while_condition, parse_action)),
        move |(condition, body)| Statement::While {
            condition: condition,
            body: body,
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
