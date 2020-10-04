use nom::{character::complete::multispace0, combinator::map, sequence::tuple, IResult};

use crate::{
    action::{parse_action, Action},
    basic_types::MutableContext,
    function::Functions,
    pattern::{parse_item_pattern, Pattern},
    printable::Printable,
};

pub(crate) struct Item {
    pattern: Pattern,
    action: Action,
}

impl Item {
    pub(crate) fn output_for_line<'a>(
        &self,
        functions: &Functions,
        context: &mut MutableContext<'a>,
    ) -> Printable<()> {
        self.pattern
            .matches(functions, context)
            .and_then(|matched| {
                if matched {
                    self.action.output_for_line(functions, context)
                } else {
                    Printable::wrap(())
                }
            })
    }

    pub(crate) fn output_for_begin<'a>(
        &self,
        functions: &Functions,
        context: &mut MutableContext<'a>,
    ) -> Printable<()> {
        if let Pattern::Begin = self.pattern {
            self.action.output_for_line(functions, context)
        } else {
            Printable::wrap(())
        }
    }
}

pub(crate) fn parse_item(input: &str) -> IResult<&str, Item> {
    map(
        tuple((parse_item_pattern, multispace0, parse_action)),
        |(pattern, _, action)| Item {
            pattern: pattern,
            action: action,
        },
    )(input)
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
    fn test_full_item_parsing() {
        let (functions, mut variables) = empty_functions_and_variables();
        let mut context = MutableContext::for_variables(&mut variables);
        context.set_record_with_line("hello world today");
        let empty_string_vec: Vec<&'static str> = vec![];

        let result = parse_item(r#"$1 ~ "hello" { print($0); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result
                .unwrap()
                .1
                .output_for_line(&functions, &mut context)
                .output,
            vec!["hello world today"],
        );

        let result = parse_item(r#"$2 ~ "hello" { print($0); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result
                .unwrap()
                .1
                .output_for_line(&functions, &mut context)
                .output,
            empty_string_vec,
        );

        let result = parse_item(r#"11 ~ 1 { print($3); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result
                .unwrap()
                .1
                .output_for_line(&functions, &mut context)
                .output,
            vec!["today"],
        );
    }
}
