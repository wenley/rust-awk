use regex::Regex;

use nom::{re_find, IResult};

use super::{Assign, Expression, ExpressionParseResult};
use crate::{
    basic_types::{MutableContext, VariableStore},
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

    fn evaluate(&self, _functions: &Functions, context: &mut MutableContext) -> Value {
        context.fetch_variable(&self.variable_name)
    }
}

impl Assign for Variable {
    fn assign<'a>(&self, functions: &Functions, context: &mut MutableContext, value: Value) {
        if let Some(_) = functions.get(&self.variable_name) {
            panic!("can't assign to {}; it's a function", self.variable_name);
        }
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
    use crate::basic_types::{Record, Variables};
    use crate::function::{parse_function, Functions};
    use crate::value::NumericValue;
    use std::collections::HashMap;

    fn empty_variables_and_record() -> (Functions, Variables, Record<'static>) {
        let variables = Variables::empty();
        let record = variables.record_for_line("");
        (HashMap::new(), variables, record)
    }

    #[test]
    fn variables_can_evaluate() {
        let (functions, mut variables, record) = empty_variables_and_record();
        let value = Value::Numeric(NumericValue::Integer(1));
        variables.assign_variable("foo", value.clone());
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record(&record);

        assert_eq!(
            Variable {
                variable_name: "foo".to_string()
            }
            .evaluate(&functions, &mut context),
            value,
        );
    }

    #[test]
    #[should_panic]
    fn blocks_name_collisions() {
        let (mut functions, mut variables, record) = empty_variables_and_record();
        let value = Value::Numeric(NumericValue::Integer(1));
        let result = parse_function(r#"function foo(a) {}"#);
        assert!(result.is_ok());
        let function = result.unwrap().1;
        functions.insert("foo".to_string(), function);

        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record(&record);

        let result = parse_assignable_variable("foo");
        assert!(result.is_ok());
        let assignment = result.unwrap().1;

        assignment.assign(&functions, &mut context, value);
    }
}
