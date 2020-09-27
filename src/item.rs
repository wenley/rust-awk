use nom::{
    character::complete::multispace0,
    combinator::map,
    multi::many1,
    sequence::{delimited, tuple},
    IResult,
};

use crate::{
    action::{parse_action, Action},
    basic_types::{Context, Record},
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
        context: &mut Context,
        record: &Record<'a>,
    ) -> Vec<String> {
        if self.pattern.matches(functions, context, record) {
            self.action.output_for_line(functions, context, record)
        } else {
            vec![]
        }
    }

    pub(crate) fn output_for_begin(
        &self,
        functions: &Functions,
        context: &mut Context,
    ) -> Vec<String> {
        if let Pattern::Begin = self.pattern {
            let empty_record = Record {
                full_line: "",
                fields: vec![],
            };
            self.action
                .output_for_line(functions, context, &empty_record)
        } else {
            vec![]
        }
    }
}

pub(crate) fn parse_item_list(input: &str) -> IResult<&str, Vec<Item>> {
    many1(delimited(multispace0, parse_item, multispace0))(input)
}

fn parse_item(input: &str) -> IResult<&str, Item> {
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
    use crate::function::Functions;
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
    fn test_full_item_parsing() {
        let (functions, mut context, _) = empty_context_and_record();
        let record = Record {
            full_line: "hello world today",
            fields: vec!["hello", "world", "today"],
        };
        let empty_string_vec: Vec<&'static str> = vec![];

        let result = parse_item(r#"$1 ~ "hello" { print($0); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result
                .unwrap()
                .1
                .output_for_line(&functions, &mut context, &record),
            vec!["hello world today"],
        );

        let result = parse_item(r#"$2 ~ "hello" { print($0); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result
                .unwrap()
                .1
                .output_for_line(&functions, &mut context, &record),
            empty_string_vec,
        );

        let result = parse_item(r#"11 ~ 1 { print($3); }"#);
        assert!(result.is_ok());
        assert_eq!(
            result
                .unwrap()
                .1
                .output_for_line(&functions, &mut context, &record),
            vec!["today"],
        );
    }
}
