use nom::{
    branch::alt,
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::many0,
    sequence::{delimited, terminated, tuple},
    IResult,
};

use crate::{context::MutableContext, function::Functions, printable::Printable};

mod assign;
mod do_while;
mod if_else;
mod print;
mod while_statement;

pub(crate) struct Action {
    statements: Vec<Box<dyn Statement>>,
}

impl Action {
    pub(crate) fn output_for_line(
        &self,
        functions: &Functions,
        context: &mut MutableContext,
    ) -> Printable<()> {
        self.statements
            .iter()
            .fold(Printable::wrap(()), |result, statement| {
                result.and_then(|_| statement.evaluate(functions, context))
            })
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

trait Statement {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<()>;
}

fn parse_statements(input: &str) -> IResult<&str, Vec<Box<dyn Statement>>> {
    let parse_single_statement = terminated(
        parse_simple_statement,
        tuple((multispace0, one_of(";"), multispace0)),
    );
    many0(parse_single_statement)(input)
}

fn parse_simple_statement(input: &str) -> IResult<&str, Box<dyn Statement>> {
    alt((
        print::parse_print_statement,
        if_else::parse_if_else_statement,
        while_statement::parse_while_statement,
        do_while::parse_do_while_statement,
        assign::parse_assign_statement,
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::empty_functions_and_variables;

    #[test]
    fn test_parse_action() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);

        let result = parse_action(r#"{ print("hello"); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result
                .unwrap()
                .1
                .output_for_line(&functions, &mut context)
                .output,
            vec!["hello"],
        );

        let result = parse_action(
            r#"{
            print(1);
            print(2.0   ,    "extra arg");
            print("hello");
        }"#,
        );
        assert!(result.is_ok());
        assert_eq!(
            result
                .unwrap()
                .1
                .output_for_line(&functions, &mut context)
                .output,
            vec!["1", "2 extra arg", "hello",],
        );
    }
}
