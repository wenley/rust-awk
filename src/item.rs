use nom::{character::complete::multispace0, combinator::map, sequence::tuple, IResult};

use crate::{
    action::{parse_action, Action},
    basic_types::{MutableContext, Variables},
    function::Functions,
    pattern::{parse_item_pattern, Pattern},
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
    ) -> Vec<String> {
        if self.pattern.matches(functions, context) {
            self.action.output_for_line(functions, context)
        } else {
            vec![]
        }
    }

    pub(crate) fn output_for_begin(
        &self,
        functions: &Functions,
        variables: &mut Variables,
    ) -> Vec<String> {
        if let Pattern::Begin = self.pattern {
            let record = variables.record_for_line("");
            let mut context = MutableContext {
                variables: variables,
                record: Some(&record),
            };
            self.action.output_for_line(functions, &mut context)
        } else {
            vec![]
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
    use crate::basic_types::Record;
    use crate::function::Functions;
    use std::collections::HashMap;

    fn empty_variables_and_record() -> (Functions, Variables, Record<'static>) {
        let variables = Variables::empty();
        let record = variables.record_for_line("");
        (HashMap::new(), variables, record)
    }

    #[test]
    fn test_full_item_parsing() {
        let (functions, mut variables, _) = empty_variables_and_record();
        let record = variables.record_for_line("hello world today");
        let mut context = MutableContext {
            variables: &mut variables,
            record: Some(&record),
        };
        let empty_string_vec: Vec<&'static str> = vec![];

        let result = parse_item(r#"$1 ~ "hello" { print($0); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.output_for_line(&functions, &mut context),
            vec!["hello world today"],
        );

        let result = parse_item(r#"$2 ~ "hello" { print($0); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.output_for_line(&functions, &mut context),
            empty_string_vec,
        );

        let result = parse_item(r#"11 ~ 1 { print($3); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1.output_for_line(&functions, &mut context),
            vec!["today"],
        );
    }
}
