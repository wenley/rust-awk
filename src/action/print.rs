use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, one_of},
    multi::separated_list,
    sequence::delimited,
    IResult,
};

use super::Statement;
use crate::{
    context::{MutableContext, VariableStore},
    expression::{parse_expression, Expression},
    function::Functions,
    printable::Printable,
};

struct Print {
    expressions: Vec<Box<dyn Expression>>,
}

impl Statement for Print {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<()> {
        self.expressions
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
                output: vec![strings.join(&context.fetch_variable("OFS").coerce_to_string())],
            })
    }
}

pub(super) fn parse_print_statement(input: &str) -> IResult<&str, Box<dyn Statement>> {
    let parse_separator = delimited(multispace0, one_of(","), multispace0);
    let parse_expression_list = separated_list(parse_separator, parse_expression);

    let (i, exprs) = delimited(tag("print("), parse_expression_list, one_of(")"))(input)?;
    Result::Ok((i, Box::new(Print { expressions: exprs })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::empty_functions_and_variables;

    #[test]
    fn print_statement_produces_value() {
        let (functions, mut empty_variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut empty_variables);

        let print_statement = parse_print_statement(r#"print("hello")"#).unwrap().1;
        assert_eq!(
            print_statement.evaluate(&functions, &mut context).output,
            vec!["hello"],
        );
    }
}
