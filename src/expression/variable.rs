use regex::Regex;

use nom::{re_find, IResult};

use super::{Assign, Expression, ExpressionParseResult};
use crate::{
    basic_types::{MutableContext, VariableStore},
    function::Functions,
    printable::Printable,
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

    fn evaluate(&self, _functions: &Functions, context: &mut MutableContext) -> Printable<Value> {
        Printable::wrap(context.fetch_variable(&self.variable_name))
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
    use crate::function::parse_function;
    use crate::test_utilities::empty_functions_and_variables;
    use crate::value::NumericValue;

    #[test]
    fn variables_can_evaluate() {
        let (functions, mut variables) = empty_functions_and_variables();
        let value = Value::Numeric(NumericValue::Integer(1));
        variables.assign_variable("foo", value.clone());
        let mut context = MutableContext::for_variables(&mut variables);

        assert_eq!(
            Variable {
                variable_name: "foo".to_string()
            }
            .evaluate(&functions, &mut context)
            .value,
            value,
        );
    }

    #[test]
    #[should_panic]
    fn blocks_name_collisions() {
        let (mut functions, mut variables) = empty_functions_and_variables();
        let value = Value::Numeric(NumericValue::Integer(1));
        let result = parse_function(r#"function foo(a) {}"#);
        assert!(result.is_ok());
        let function = result.unwrap().1;
        functions.insert("foo".to_string(), function);

        let mut context = MutableContext::for_variables(&mut variables);

        let result = parse_assignable_variable("foo");
        assert!(result.is_ok());
        let assignment = result.unwrap().1;

        assignment.assign(&functions, &mut context, value);
    }
}
