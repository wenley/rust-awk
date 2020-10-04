use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, one_of},
    sequence::tuple,
    IResult,
};

use super::{parse_action, Action, Statement};
use crate::{
    basic_types::MutableContext,
    expression::{parse_expression, Expression},
    function::Functions,
    printable::Printable,
    value::UNINITIALIZED_VALUE,
};

struct DoWhile {
    body: Action,
    condition: Box<dyn Expression>,
}

impl Statement for DoWhile {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<()> {
        let mut result = Printable::wrap(UNINITIALIZED_VALUE.clone());
        loop {
            result = result
                .and_then(|_| self.body.output_for_line(functions, context))
                .and_then(|_| self.condition.evaluate(functions, context));
            if !result.value.coercion_to_boolean() {
                break;
            }
        }
        result.map(|_| ())
    }
}

pub(super) fn parse_do_while_statement(input: &str) -> IResult<&str, Box<dyn Statement>> {
    let (i, (_, _, body, _, _, _, _, condition, _)) = tuple((
        tag("do"),
        multispace0,
        parse_action,
        multispace0,
        tag("while"),
        multispace0,
        one_of("("),
        parse_expression,
        one_of(")"),
    ))(input)?;
    Result::Ok((
        i,
        Box::new(DoWhile {
            body: body,
            condition: condition,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::empty_functions_and_variables;

    #[test]
    fn test_parse_do_while_statement() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);

        let result = parse_do_while_statement(
            r#"do {
                print("hello");
            } while (0)"#,
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).output,
            vec!["hello"],
        );
    }
}
