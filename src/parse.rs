extern crate nom;
extern crate regex;

use nom::{
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::many1,
    sequence::{delimited, pair, tuple},
    IResult,
};

use crate::{
    expression::Expression,
    item::{Action, Item},
    pattern::{parse_item_pattern, Pattern},
    statement::{parse_statements, Statement},
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
