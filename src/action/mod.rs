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
    basic_types::MutableContext,
    expression::{parse_assignable, parse_expression, Assign, Expression},
    function::Functions,
    printable::Printable,
    value::UNINITIALIZED_VALUE,
};

#[derive(Debug)]
pub(crate) struct Action {
    statements: Vec<StatementEnum>,
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

trait StateTraitMent {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<()>;
}

#[derive(Debug)]
enum StatementEnum {
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

impl StateTraitMent for StatementEnum {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<()> {
        match self {
            StatementEnum::Print(expressions) => expressions
                .iter()
                .fold(Printable::wrap(vec![]), |result, e| {
                    result.and_then(|mut vec| {
                        e.evaluate(functions, context).map(|value| {
                            vec.push(value.coerce_to_string());
                            vec
                        })
                    })
                })
                .and_then(|strings| Printable {
                    value: (),
                    output: vec![strings.join(" ")],
                }),
            StatementEnum::IfElse {
                condition,
                if_branch,
                else_branch,
            } => condition.evaluate(functions, context).and_then(|value| {
                if value.coercion_to_boolean() {
                    if_branch.output_for_line(functions, context)
                } else {
                    else_branch.output_for_line(functions, context)
                }
            }),
            StatementEnum::Assign { assignable, value } => {
                value.evaluate(functions, context).map(|value| {
                    assignable.assign(functions, context, value);
                    ()
                })
            }
            StatementEnum::While { condition, body } => {
                let mut result = condition.evaluate(functions, context);
                loop {
                    if result.value.coercion_to_boolean() {
                        result = result
                            .and_then(|_| body.output_for_line(functions, context))
                            .and_then(|_| condition.evaluate(functions, context));
                    } else {
                        break;
                    }
                }
                result.map(|_| ())
            }
            StatementEnum::DoWhile { body, condition } => {
                let mut result = Printable::wrap(UNINITIALIZED_VALUE.clone());
                loop {
                    result = result
                        .and_then(|_| body.output_for_line(functions, context))
                        .and_then(|_| condition.evaluate(functions, context));
                    if !result.value.coercion_to_boolean() {
                        break;
                    }
                }
                result.map(|_| ())
            }
        }
    }
}

fn parse_statements(input: &str) -> IResult<&str, Vec<StatementEnum>> {
    let parse_single_statement = terminated(
        parse_simple_statement,
        tuple((multispace0, one_of(";"), multispace0)),
    );
    many0(parse_single_statement)(input)
}

fn parse_simple_statement(input: &str) -> IResult<&str, StatementEnum> {
    alt((
        parse_print_statement,
        parse_if_else_statement,
        parse_while_statement,
        parse_do_while_statement,
        parse_assign_statement,
        map(parse_expression, move |expr| StatementEnum::Print(vec![expr])),
    ))(input)
}

fn parse_print_statement(input: &str) -> IResult<&str, StatementEnum> {
    let parse_separator = delimited(multispace0, one_of(","), multispace0);
    let parse_expression_list = separated_list(parse_separator, parse_expression);
    map(
        delimited(tag("print("), parse_expression_list, one_of(")")),
        move |exprs| StatementEnum::Print(exprs),
    )(input)
}

fn parse_if_else_statement(input: &str) -> IResult<&str, StatementEnum> {
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

    map(
        tuple((
            parse_if,
            multispace0,
            parse_action,
            multispace0,
            tag("else"),
            multispace0,
            parse_action,
        )),
        move |(condition, _, if_branch, _, _, _, else_branch)| StatementEnum::IfElse {
            condition: condition,
            if_branch: if_branch,
            else_branch: else_branch,
        },
    )(input)
}

fn parse_while_statement(input: &str) -> IResult<&str, StatementEnum> {
    map(
        tuple((
            tag("while"),
            multispace0,
            one_of("("),
            multispace0,
            parse_expression,
            multispace0,
            one_of(")"),
            multispace0,
            parse_action,
        )),
        move |(_, _, _, _, condition, _, _, _, body)| StatementEnum::While {
            condition: condition,
            body: body,
        },
    )(input)
}

fn parse_do_while_statement(input: &str) -> IResult<&str, StatementEnum> {
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
        |(_, _, body, _, _, _, _, condition, _)| StatementEnum::DoWhile {
            body: body,
            condition: condition,
        },
    )(input)
}

fn parse_assign_statement(input: &str) -> IResult<&str, StatementEnum> {
    map(
        tuple((
            parse_assignable,
            multispace0,
            one_of("="),
            multispace0,
            parse_expression,
        )),
        |(assignable, _, _, _, value_expression)| StatementEnum::Assign {
            assignable: assignable,
            value: value_expression,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::{VariableStore, Variables};
    use crate::function::Functions;
    use crate::value::{NumericValue, Value};
    use std::collections::HashMap;

    fn empty_functions_and_variables() -> (Functions, Variables) {
        let variables = Variables::empty();
        (HashMap::new(), variables)
    }

    #[test]
    fn print_statement_produces_value() {
        let (functions, mut empty_variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut empty_variables);
        context.set_record_with_line("");

        let print_action = parse_action(r#"{ print("hello"); }"#).unwrap().1;
        assert_eq!(
            print_action
                .output_for_line(&functions, &mut context)
                .output,
            vec!["hello"],
        );
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
    fn assignment_updates_variables() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

        let assign_action = parse_action(
            r#"{
            foo = 1 + 2;
        }"#,
        )
        .unwrap()
        .1;
        assign_action.output_for_line(&functions, &mut context);
        assert_eq!(
            context.fetch_variable("foo"),
            Value::Numeric(NumericValue::Integer(3)),
        );
    }

    #[test]
    fn test_parse_statements() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

        let result = parse_print_statement(r#"print("hello")"#);
        assert!(result.is_ok());
        assert_eq!(
            Action {
                statements: vec![result.unwrap().1]
            }
            .output_for_line(&functions, &mut context)
            .output,
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
            .output_for_line(&functions, &mut context)
            .output,
            vec!["1", "2 extra arg", "hello",],
        );
    }

    #[test]
    fn test_parse_if_else_statement() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

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
            .output_for_line(&functions, &mut context)
            .output,
            vec!["hello"],
        );
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

    #[test]
    fn test_parse_do_while_statement() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

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
            .output_for_line(&functions, &mut context)
            .output,
            vec!["hello"],
        );
    }
    #[test]
    fn test_parse_assign_statement() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("");

        let result = parse_simple_statement(r#"variable = "hi""#);
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

    #[test]
    fn test_assign_from_function() {
        let result = parse_simple_statement(r#"variable = hello("hi")"#);
        assert!(result.is_ok());
        let (remaining, _statement) = result.unwrap();
        assert_eq!(remaining, "");
    }
}
