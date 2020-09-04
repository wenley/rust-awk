use nom::{combinator::map, multi::many1, sequence::pair, IResult};

use crate::{
    action::{parse_action, Action},
    basic_types::{Context, Record},
    pattern::{parse_item_pattern, Pattern},
};

pub(crate) struct Item {
    pattern: Pattern,
    action: Action,
}

impl Item {
    pub(crate) fn output_for_line<'a>(
        &self,
        context: &mut Context,
        record: &Record<'a>,
    ) -> Vec<String> {
        if self.pattern.matches(context, record) {
            self.action.output_for_line(context, record)
        } else {
            vec![]
        }
    }

    pub(crate) fn output_for_begin(&self, context: &mut Context) -> Vec<String> {
        if let Pattern::Begin = self.pattern {
            let empty_record = Record {
                full_line: "",
                fields: vec![],
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

    fn empty_context_and_record() -> (Context, Record<'static>) {
        (
            Context::empty(),
            Record {
                full_line: "",
                fields: vec![],
            },
        )
    }
}
