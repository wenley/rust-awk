use nom::{
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::many1,
    sequence::{delimited, pair, tuple},
    IResult,
};

use crate::{
    basic_types::{Context, Record},
    pattern::{parse_item_pattern, Pattern},
    statement::{parse_statements, Statement},
};

pub struct Action {
    pub(crate) statements: Vec<Statement>,
}

impl Action {
    pub fn output_for_line<'a>(&self, context: &mut Context, record: &Record<'a>) -> Vec<String> {
        self.statements
            .iter()
            .map(|statement| statement.evaluate(context, record))
            .collect()
    }
}

pub struct Item {
    pub(crate) pattern: Pattern,
    pub action: Action,
}

pub(crate) fn parse_item_list(input: &str) -> IResult<&str, Vec<Item>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        basic_types::Record,
        expression::Expression,
        value::{NumericValue, Value},
    };

    #[test]
    fn print_statement_produces_value() {
        let mut empty_context = Context::empty();
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };
        let print_statement =
            Statement::Print(vec![Expression::StringLiteral("hello".to_string())]);
        assert_eq!(
            print_statement.evaluate(&mut empty_context, &record),
            "hello",
        );
    }

    #[test]
    fn if_produces_correct_value() {
        let mut empty_context = Context::empty();
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };

        let if_conditional = Statement::IfElse {
            condition: Expression::StringLiteral("not empty".to_string()),
            if_branch: Box::new(Statement::Print(vec![Expression::StringLiteral(
                "if-branch".to_string(),
            )])),
            else_branch: Box::new(Statement::Print(vec![Expression::StringLiteral(
                "else".to_string(),
            )])),
        };
        assert_eq!(
            if_conditional.evaluate(&mut empty_context, &record),
            "if-branch",
        );

        let else_conditional = Statement::IfElse {
            condition: Expression::StringLiteral("".to_string()),
            if_branch: Box::new(Statement::Print(vec![Expression::StringLiteral(
                "if-branch".to_string(),
            )])),
            else_branch: Box::new(Statement::Print(vec![Expression::StringLiteral(
                "else".to_string(),
            )])),
        };
        assert_eq!(
            else_conditional.evaluate(&mut empty_context, &record),
            "else",
        );
    }

    #[test]
    fn assignment_updates_context() {
        let mut context = Context::empty();
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };

        let assign = Statement::Assign {
            variable_name: "foo".to_string(),
            value: Expression::AddBinary {
                left: Box::new(Expression::NumericLiteral(NumericValue::Integer(1))),
                right: Box::new(Expression::NumericLiteral(NumericValue::Integer(2))),
            },
        };
        assign.evaluate(&mut context, &record);
        assert_eq!(
            context.fetch_variable("foo"),
            Value::Numeric(NumericValue::Integer(3)),
        );
    }
}
