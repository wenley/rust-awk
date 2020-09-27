use regex::Regex;

use nom::{re_find, IResult};

use super::{Assign, Expression, ExpressionParseResult};
use crate::{basic_types::MutableContext, function::Functions, value::Value};

#[derive(Debug)]
struct Variable {
    variable_name: String,
}

impl Expression for Variable {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate(&self, _functions: &Functions, context: &mut MutableContext) -> Value {
        context.variables.fetch_variable(&self.variable_name)
    }
}

impl Assign for Variable {
    fn assign<'a>(&self, context: &mut MutableContext, value: Value) {
        context
            .variables
            .assign_variable(&self.variable_name, value);
    }
}

pub(super) fn parse_variable(input: &str) -> ExpressionParseResult {
    let (i, name) = parse_variable_name(input)?;

    Result::Ok((
        i,
        Box::new(Variable {
            variable_name: name.to_string(),
        }),
    ))
}

pub(super) fn parse_assignable_variable(input: &str) -> IResult<&str, Box<dyn Assign>> {
    let (i, name) = parse_variable_name(input)?;

    Result::Ok((
        i,
        Box::new(Variable {
            variable_name: name.to_string(),
        }),
    ))
}

// Public for use in parse_args
pub fn parse_variable_name(input: &str) -> IResult<&str, &str> {
    re_find!(input, r"^[A-Za-z_][A-Za-z0-9_]*")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::{Record, Variables};
    use crate::function::Functions;
    use crate::value::NumericValue;
    use std::collections::HashMap;

    fn empty_variables_and_record() -> (Functions, Variables, Record<'static>) {
        (
            HashMap::new(),
            Variables::empty(),
            Record {
                full_line: "",
                fields: vec![],
            },
        )
    }

    #[test]
    fn variables_can_evaluate() {
        let (functions, mut variables, record) = empty_variables_and_record();
        let value = Value::Numeric(NumericValue::Integer(1));
        variables.assign_variable("foo", value.clone());
        let mut context = MutableContext {
            variables: &mut variables,
            record: &record,
        };

        assert_eq!(
            Variable {
                variable_name: "foo".to_string()
            }
            .evaluate(&functions, &mut context),
            value,
        );
    }
}
