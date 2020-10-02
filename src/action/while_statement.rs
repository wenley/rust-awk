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
};

struct While {
    condition: Box<dyn Expression>,
    body: Action,
}

impl Statement for While {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<()> {
        let mut result = self.condition.evaluate(functions, context);
        loop {
            if result.value.coercion_to_boolean() {
                result = result
                    .and_then(|_| self.body.output_for_line(functions, context))
                    .and_then(|_| self.condition.evaluate(functions, context));
            } else {
                break;
            }
        }
        result.map(|_| ())
    }
}

pub(super) fn parse_while_statement(input: &str) -> IResult<&str, Box<dyn Statement>> {
    let (i, (_, _, _, _, condition, _, _, _, body)) = tuple((
        tag("while"),
        multispace0,
        one_of("("),
        multispace0,
        parse_expression,
        multispace0,
        one_of(")"),
        multispace0,
        parse_action,
    ))(input)?;

    Result::Ok((
        i,
        Box::new(While {
            condition: condition,
            body: body,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::parse_simple_statement;
    use crate::basic_types::Variables;
    use crate::function::Functions;
    use std::collections::HashMap;

    fn empty_functions_and_variables() -> (Functions, Variables) {
        let variables = Variables::empty();
        (HashMap::new(), variables)
    }

    #[test]
    fn test_parse_while_statement() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

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
            .output_for_line(&functions, &mut context)
            .output,
            empty_vec,
        );
    }
}
