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
    expression::{parse_assignable, parse_expression, Assign, Expression},
};

#[derive(Debug)]
pub(crate) struct Action {
    statements: Vec<Statement>,
}

impl Action {
    pub(crate) fn output_for_line<'a>(
        &self,
        context: &mut Context,
        record: &Record<'a>,
    ) -> Vec<String> {
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

#[derive(Debug)]
enum Statement {
    IfElse {
        condition: Box<dyn Expression>,
        if_branch: Action,
        else_branch: Action,
    },
    Print(Vec<Box<dyn Expression>>),
    Assign {
        assignable: Box<dyn Assign>,
        value: Box<dyn Expression>,
    },
    While {
        condition: Box<dyn Expression>,
        body: Action,
    },
    DoWhile {
        body: Action,
        condition: Box<dyn Expression>,
    },
}

impl Statement {
    fn evaluate<'a>(&self, context: &mut Context, record: &'a Record) -> Vec<String> {
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
            Statement::Assign { assignable, value } => {
                let value = value.evaluate(context, record);
                assignable.assign(context, record, value);
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
            Statement::DoWhile { body, condition } => {
                let mut output = vec![];
                loop {
                    output.append(&mut body.output_for_line(context, record));
                    let value = condition.evaluate(context, record);
                    if !value.coercion_to_boolean() {
                        break;
                    }
                }
                output
            }
        }
    }
}

fn parse_statements(input: &str) -> IResult<&str, Vec<Statement>> {
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
        parse_while_statement,
        parse_do_while_statement,
        parse_assign_statement,
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

fn parse_do_while_statement(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("do"),
            multispace0,
            parse_action,
            multispace0,
            tag("while"),
            multispace0,
            one_of("("),
            parse_expression,
            one_of(")"),
        )),
        |(_, _, body, _, _, _, _, condition, _)| Statement::DoWhile {
            body: body,
            condition: condition,
        },
    )(input)
}

fn parse_assign_statement(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            parse_assignable,
            multispace0,
            one_of("="),
            multispace0,
            parse_expression,
        )),
        |(assignable, _, _, _, value_expression)| Statement::Assign {
            assignable: assignable,
            value: value_expression,
        },
    )(input)
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
    fn test_parse_statements() {
        let (mut context, record) = empty_context_and_record();
        let result = parse_print_statement(r#"print("hello")"#);
        assert!(result.is_ok());
        assert_eq!(
            Action {
                statements: vec![result.unwrap().1]
            }
            .output_for_line(&mut context, &record),
            vec!["hello"],
        );

        let result = parse_statements(
            r#"print(1);
            print(2.0   ,    "extra arg");
            print("hello");
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(
            Action {
                statements: result.unwrap().1
            }
            .output_for_line(&mut context, &record),
            vec!["1", "2 extra arg", "hello",],
        );
    }

    #[test]
    fn test_parse_if_else_statement() {
        let (mut context, record) = empty_context_and_record();
        let result = parse_simple_statement(
            r#"if (1) {
            print("hello");
        } else {}"#,
        );
        assert!(result.is_ok());
        assert_eq!(
            Action {
                statements: vec![result.unwrap().1]
            }
            .output_for_line(&mut context, &record),
            vec!["hello"],
        );
    }

    #[test]
    fn test_parse_while_statement() {
        let (mut context, record) = empty_context_and_record();
        let result = parse_simple_statement(
            r#"while (0) {
                print("hello");
            }"#,
        );
        let empty_vec: Vec<&'static str> = vec![];
        assert!(result.is_ok());
        assert_eq!(
            Action {
                statements: vec![result.unwrap().1]
            }
            .output_for_line(&mut context, &record),
            empty_vec,
        );
    }

    #[test]
    fn test_parse_do_while_statement() {
        let (mut context, record) = empty_context_and_record();
        let result = parse_simple_statement(
            r#"do {
                print("hello");
            } while (0)"#,
        );
        assert!(result.is_ok());
        assert_eq!(
            Action {
                statements: vec![result.unwrap().1]
            }
            .output_for_line(&mut context, &record),
            vec!["hello"],
        );
    }
    #[test]
    fn test_parse_assign_statement() {
        let (mut context, record) = empty_context_and_record();
        let result = parse_simple_statement(r#"variable = "hi""#);
        let empty_vec: Vec<&'static str> = vec![];
        assert!(result.is_ok());
        assert_eq!(
            Action {
                statements: vec![result.unwrap().1]
            }
            .output_for_line(&mut context, &record),
            empty_vec,
        );
    }
}
