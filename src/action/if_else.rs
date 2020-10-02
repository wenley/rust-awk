use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, one_of},
    combinator::map,
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

struct IfElse {
    condition: Box<dyn Expression>,
    if_branch: Action,
    else_branch: Action,
}

impl Statement for IfElse {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<()> {
        self.condition
            .evaluate(functions, context)
            .and_then(|value| {
                if value.coercion_to_boolean() {
                    self.if_branch.output_for_line(functions, context)
                } else {
                    self.else_branch.output_for_line(functions, context)
                }
            })
    }
}

pub(super) fn parse_if_else_statement(input: &str) -> IResult<&str, Box<dyn Statement>> {
    let parse_if = map(
        tuple((
            tag("if"),
            multispace0,
            one_of("("),
            multispace0,
            parse_expression,
            multispace0,
            one_of(")"),
        )),
        |(_, _, _, _, expression, _, _)| expression,
    );

    let (i, (condition, _, if_branch, _, _, _, else_branch)) = tuple((
        parse_if,
        multispace0,
        parse_action,
        multispace0,
        tag("else"),
        multispace0,
        parse_action,
    ))(input)?;
    Result::Ok((
        i,
        Box::new(IfElse {
            condition: condition,
            if_branch: if_branch,
            else_branch: else_branch,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::Variables;
    use crate::function::Functions;
    use std::collections::HashMap;

    fn empty_functions_and_variables() -> (Functions, Variables) {
        let variables = Variables::empty();
        (HashMap::new(), variables)
    }

    #[test]
    fn if_produces_correct_value() {
        let (functions, mut empty_variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut empty_variables);
        context.set_record_with_line("");

        let if_conditional = parse_action(
            r#"{
            if ("not empty") {
                print("if-branch");
            } else {
                print("else");
            };
        }"#,
        )
        .unwrap()
        .1;
        assert_eq!(
            if_conditional
                .output_for_line(&functions, &mut context)
                .output,
            vec!["if-branch"],
        );

        let else_conditional = parse_action(
            r#"{
            if ("") {
                print("if-branch");
            } else {
                print("else");
            };
        }"#,
        )
        .unwrap()
        .1;
        assert_eq!(
            else_conditional
                .output_for_line(&functions, &mut context)
                .output,
            vec!["else"],
        );
    }

    #[test]
    fn test_parse_if_else_statement() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

        let result = parse_if_else_statement(
            r#"if (1) {
            print("hello");
        } else {}"#,
        );
        assert!(result.is_ok());
        assert_eq!(
            Action {
                statements: vec![result.unwrap().1]
            }
            .output_for_line(&functions, &mut context)
            .output,
            vec!["hello"],
        );
    }
}
