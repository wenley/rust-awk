extern crate nom;
extern crate regex;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, one_of},
    combinator::{map},
    multi::{many0, many1},
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};

use crate::{
    expression::{parse_expression, Expression},
    item::{Action, Item, Statement},
    pattern::{Pattern, parse_item_pattern},
};

pub struct Program {
    pub items: Vec<Item>,
}

/* - - - - - - - - - -
 * Statement Parsers
 * - - - - - - - - - - */

fn parse_item_list(input: &str) -> IResult<&str, Vec<Item>> {
    many1(parse_item)(input)
}

fn parse_item(input: &str) -> IResult<&str, Item> {
    map(
        pair(parse_item_pattern, parse_action),
        |(pattern, action)| Item {
            pattern: pattern,
            action: action,
        },
    )(input)
}

fn parse_action(input: &str) -> IResult<&str, Action> {
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
        map(parse_expression, move |expr| Statement::Print(expr)),
    ))(input)
}

fn parse_print_statement(input: &str) -> IResult<&str, Statement> {
    map(
        delimited(tag("print("), parse_expression, one_of(")")),
        move |expr| Statement::Print(expr),
    )(input)
}

pub fn parse_program(program_text: &str) -> Program {
    let default_program = Program {
        items: vec![Item {
            pattern: Pattern::MatchEverything,
            action: Action {
                statements: vec![Statement::Print(Expression::StringLiteral(
                    "hi".to_string(),
                ))],
            },
        }],
    };

    match parse_item_list(program_text) {
        Ok((_, items)) => Program { items: items },
        _ => default_program,
    }
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
            Statement::Print(Expression::StringLiteral("hello".to_string()))
        );

        let result = parse_statements(
            r#"print(1);
            print(2.0);
            print("hello");
        "#,
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            vec![
                Statement::Print(Expression::NumericLiteral(NumericValue::Integer(1))),
                Statement::Print(Expression::NumericLiteral(NumericValue::Float(2.0))),
                Statement::Print(Expression::StringLiteral("hello".to_string())),
            ],
        );
    }

    #[test]
    fn test_parse_program() {
        let program = parse_program(
            r#"{ print(1);
            print(2.0);
            print("hello");
        }"#,
        );

        assert_eq!(program.items[0].action.statements.len(), 3);
    }
}
