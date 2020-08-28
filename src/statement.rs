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

static EMPTY_STRING: &str = "";

#[derive(PartialEq, Debug)]
pub(crate) enum Statement {
    IfElse {
        condition: Expression,
        if_branch: Box<Statement>,
        else_branch: Box<Statement>,
    },
    Print(Vec<Expression>),
    Assign {
        variable_name: String,
        value: Expression,
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
}
