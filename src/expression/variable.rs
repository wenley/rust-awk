use regex::Regex;

use nom::{
    character::complete::alpha1,
    IResult,
};

use crate::{
    basic_types::{Context, Record},
    value::Value,
};
use super::Expression;

#[derive(Debug)]
struct Variable {
    variable_name: String,
}

impl Expression for Variable {
    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }

    fn evaluate<'a>(&self, context: &Context, _record: &'a Record) -> Value {
        context.fetch_variable(&self.variable_name)
    }
}

pub(super) fn parse_variable(input: &str) -> IResult<&str, Box<dyn Expression>> {
    let (i, name) = alpha1(input)?;

    Result::Ok((i, Box::new(Variable { variable_name: name.to_string() })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::NumericValue;

    fn empty_context_and_record() -> (Context, Record<'static>) {
        (
            Context::empty(),
            Record {
                full_line: "",
                fields: vec![],
            },
        )
    }

    #[test]
    fn variables_can_evaluate() {
        let (mut context, record) = empty_context_and_record();
        let value = Value::Numeric(NumericValue::Integer(1));
        context.assign_variable("foo", value.clone());

        assert_eq!(
            Variable { variable_name: "foo".to_string() }.evaluate(&context, &record),
            value,
        );
    }
}
