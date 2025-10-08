use regex::Regex;
use std::fmt::Debug;

use nom::{
    character::complete::{multispace0, one_of},
    combinator::map,
    multi::many0,
    sequence::{pair, preceded, tuple},
};

use super::{parse_expression, variable::parse_variable_name, Expression, ExpressionParseResult};
use crate::{context::MutableContext, function::Functions, printable::Printable, value::{Value, NumericValue}};

#[derive(Debug)]
struct FunctionCall {
    name: String,
    arguments: Vec<Box<dyn Expression>>,
}

impl Expression for FunctionCall {
    fn evaluate(&self, functions: &Functions, context: &mut MutableContext) -> Printable<Value> {
        // Evaluate arguments first
        let args_result = self.arguments
            .iter()
            .fold(Printable::wrap(vec![]), |printable, argument| {
                printable.and_then(|mut vec| {
                    let Printable { value: v, output } = argument.evaluate(functions, context);
                    vec.push(v);
                    Printable {
                        value: vec,
                        output: output,
                    }
                })
            });

        // Check for built-in functions first
        if let Some(result) = evaluate_builtin_function(&self.name, &args_result.value) {
            return Printable {
                value: result,
                output: args_result.output,
            };
        }

        // Fall back to user-defined functions
        let function = match functions.get(&self.name) {
            Some(func) => func,
            None => panic!("Could not find function with name {}", self.name),
        };

        args_result.and_then(|values| function.invoke_with(values, functions, context))
    }

    fn regex<'a>(&'a self) -> Option<&'a Regex> {
        None
    }
}

/// Evaluates built-in functions, returns None if function is not a built-in
fn evaluate_builtin_function(name: &str, args: &[Value]) -> Option<Value> {
    match name {
        "cos" => {
            if args.len() != 1 {
                panic!("cos() function requires exactly 1 argument, got {}", args.len());
            }
            let arg = args[0].coerce_to_numeric();
            let radians = match arg {
                NumericValue::Integer(i) => i as f64,
                NumericValue::Float(f) => f,
            };
            Some(Value::Numeric(NumericValue::Float(radians.cos())))
        }
        _ => None,
    }
}

pub(crate) fn parse_function_call(input: &str) -> ExpressionParseResult {
    let parse_arguments = map(
        pair(
            parse_expression,
            many0(preceded(
                tuple((multispace0, one_of(","), multispace0)),
                parse_expression,
            )),
        ),
        |(argument, mut arguments)| {
            arguments.insert(0, argument);
            arguments
        },
    );

    let (i, (func_name, _, _, _, arguments, _, _)) = tuple((
        parse_variable_name,
        multispace0,
        one_of("("),
        multispace0,
        parse_arguments,
        multispace0,
        one_of(")"),
    ))(input)?;

    Result::Ok((
        i,
        Box::new(FunctionCall {
            name: func_name.to_string(),
            arguments: arguments,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{context::MutableContext, function::Functions, value::{Value, NumericValue}};
    use std::collections::HashMap;

    #[test]
    fn test_function_parsing() {
        // Assert no panic
        let result = parse_function_call(r#"foo("first", a, 1 + 2, $0)"#);
        assert!(result.is_ok());
        let (remaining, _call) = result.unwrap();
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_cos_builtin_function() {
        let functions: Functions = HashMap::new();
        let mut context = MutableContext::new();

        // Test cos(0) = 1
        let args = vec![Value::Numeric(NumericValue::Integer(0))];
        let result = evaluate_builtin_function("cos", &args);
        assert!(result.is_some());
        if let Some(Value::Numeric(NumericValue::Float(f))) = result {
            assert!((f - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected float result from cos(0)");
        }

        // Test cos(π/2) ≈ 0
        let pi_half = std::f64::consts::PI / 2.0;
        let args = vec![Value::Numeric(NumericValue::Float(pi_half))];
        let result = evaluate_builtin_function("cos", &args);
        assert!(result.is_some());
        if let Some(Value::Numeric(NumericValue::Float(f))) = result {
            assert!(f.abs() < 1e-10);
        } else {
            panic!("Expected float result from cos(π/2)");
        }

        // Test cos(π) ≈ -1
        let pi = std::f64::consts::PI;
        let args = vec![Value::Numeric(NumericValue::Float(pi))];
        let result = evaluate_builtin_function("cos", &args);
        assert!(result.is_some());
        if let Some(Value::Numeric(NumericValue::Float(f))) = result {
            assert!((f - (-1.0)).abs() < 1e-10);
        } else {
            panic!("Expected float result from cos(π)");
        }
    }

    #[test]
    #[should_panic(expected = "cos() function requires exactly 1 argument, got 0")]
    fn test_cos_no_args() {
        let args = vec![];
        evaluate_builtin_function("cos", &args);
    }

    #[test]
    #[should_panic(expected = "cos() function requires exactly 1 argument, got 2")]
    fn test_cos_too_many_args() {
        let args = vec![
            Value::Numeric(NumericValue::Float(1.0)),
            Value::Numeric(NumericValue::Float(2.0))
        ];
        evaluate_builtin_function("cos", &args);
    }
}
