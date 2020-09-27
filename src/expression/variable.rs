use regex::Regex;

use nom::{re_find, IResult};

use super::{Assign, Expression, ExpressionParseResult};
use crate::{
    basic_types::{Context, Record},
    function::Functions,
    value::Value,
};

#[derive(Debug)]
struct Variable {
    variable_name: String,
}

impl Expression for Variable {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate<'a>(
        &self,
        _functions: &Functions,
        context: &Context,
        _record: &'a Record,
    ) -> Value {
        context.fetch_variable(&self.variable_name)
    }
}

impl Assign for Variable {
    fn assign<'a>(&self, context: &mut Context, _record: &'a Record, value: Value) {
        context.assign_variable(&self.variable_name, value);
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
    use crate::function::Functions;
    use crate::value::NumericValue;
    use std::collections::HashMap;

    fn empty_context_and_record() -> (Functions, Context, Record<'static>) {
        (
            HashMap::new(),
            Context::empty(),
            Record {
                full_line: "",
                fields: vec![],
            },
        )
    }

    #[test]
    fn variables_can_evaluate() {
        let (functions, mut context, record) = empty_context_and_record();
        let value = Value::Numeric(NumericValue::Integer(1));
        context.assign_variable("foo", value.clone());

        assert_eq!(
            Variable {
                variable_name: "foo".to_string()
            }
            .evaluate(&functions, &mut context, &record),
            value,
        );
    }
}
