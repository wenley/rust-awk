use nom::{
    character::complete::{multispace0, one_of},
    sequence::tuple,
    IResult,
};

use super::Statement;
use crate::{
    basic_types::MutableContext,
    expression::{parse_assignable, parse_expression, Assign, Expression},
    function::Functions,
    printable::Printable,
};

struct AssignStatement {
    assignable: Box<dyn Assign>,
    value: Box<dyn Expression>,
}

impl Statement for AssignStatement {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<()> {
        self.value.evaluate(functions, context).map(|value| {
            self.assignable.assign(functions, context, value);
            ()
        })
    }
}

pub(super) fn parse_assign_statement(input: &str) -> IResult<&str, Box<dyn Statement>> {
    let (i, (assignable, _, _, _, value_expression)) = tuple((
        parse_assignable,
        multispace0,
        one_of("="),
        multispace0,
        parse_expression,
    ))(input)?;

    Result::Ok((
        i,
        Box::new(AssignStatement {
            assignable: assignable,
            value: value_expression,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        basic_types::VariableStore,
        test_utilities::empty_functions_and_variables,
        value::{NumericValue, Value},
    };

    #[test]
    fn assignment_updates_variables() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);

        let assign_statement = parse_assign_statement(r#"foo = 1 + 2"#).unwrap().1;
        assign_statement.evaluate(&functions, &mut context);
        assert_eq!(
            context.fetch_variable("foo"),
            Value::Numeric(NumericValue::Integer(3)),
        );
    }

    #[test]
    fn test_parse_assign_statement() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);

        let result = parse_assign_statement(r#"variable = "hi""#);
        let empty_vec: Vec<&'static str> = vec![];
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.evaluate(&functions, &mut context).output,
            empty_vec,
        );
    }

    #[test]
    fn test_assign_from_function() {
        let result = parse_assign_statement(r#"variable = hello("hi")"#);
        assert!(result.is_ok());
        let (remaining, _statement) = result.unwrap();
        assert_eq!(remaining, "");
    }
}
