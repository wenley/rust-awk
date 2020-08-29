use nom::{combinator::map, multi::many1, sequence::pair, IResult};

use crate::{
    action::{parse_action, Action},
    basic_types::{Context, Record},
    pattern::{parse_item_pattern, Pattern},
};

pub struct Item {
    pattern: Pattern,
    action: Action,
}

impl Item {
    pub fn output_for_line<'a>(&self, context: &mut Context, record: &Record<'a>) -> Vec<String> {
        if self.pattern.matches(record) {
            self.action.output_for_line(context, record)
        } else {
            vec![]
        }
    }

    pub fn output_for_begin(&self, context: &mut Context) -> Vec<String> {
        if let Pattern::Begin = self.pattern {
            let empty_fields = vec![];
            let empty_record = Record {
                full_line: "",
                fields: &empty_fields,
            };
            self.action.output_for_line(context, &empty_record)
        } else {
            vec![]
        }
    }
}

pub(crate) fn parse_item_list(input: &str) -> IResult<&str, Vec<Item>> {
    many1(parse_item)(input)
}

fn parse_item(input: &str) -> IResult<&str, Item> {
    map(
        pair(parse_item_pattern, parse_action),
        |(pattern, action)| Item {
            pattern: pattern,
            action: action,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        basic_types::Record,
        value::{NumericValue, Value},
    };

    #[test]
    fn print_statement_produces_value() {
        let mut empty_context = Context::empty();
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };
        let print_action = parse_action(r#"{ print("hello"); }"#).unwrap().1;
        assert_eq!(
            print_action.output_for_line(&mut empty_context, &record),
            vec!["hello"],
        );
    }

    #[test]
    fn if_produces_correct_value() {
        let mut empty_context = Context::empty();
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };

        let if_conditional = parse_action(r#"{
            if ("not empty") {
                print("if-branch");
            } else {
                print("else");
            };
        }"#).unwrap().1;
        assert_eq!(
            if_conditional.output_for_line(&mut empty_context, &record),
            vec!["if-branch"],
        );

        let else_conditional = parse_action(r#"{
            if ("") {
                print("if-branch");
            } else {
                print("else");
            };
        }"#).unwrap().1;
        assert_eq!(
            else_conditional.output_for_line(&mut empty_context, &record),
            vec!["else"],
        );
    }

    #[test]
    fn assignment_updates_context() {
        let mut context = Context::empty();
        let fields = vec![];
        let record = Record {
            full_line: "",
            fields: &fields,
        };

        let assign_action = parse_action(r#"{
            foo = 1 + 2;
        }"#).unwrap().1;
        assign_action.output_for_line(&mut context, &record);
        assert_eq!(
            context.fetch_variable("foo"),
            Value::Numeric(NumericValue::Integer(3)),
        );
    }
}
