use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    IResult,
};

use crate::{
    basic_types::{Record, Variables},
    expression::{parse_expression, Expression},
    function::Functions,
};

pub(crate) enum Pattern {
    MatchEverything,
    Expression(Box<dyn Expression>),
    Begin,
    End,
}

impl Pattern {
    pub(crate) fn matches<'a>(
        &self,
        functions: &Functions,
        variables: &mut Variables,
        record: &Record<'a>,
    ) -> bool {
        match self {
            Pattern::MatchEverything => true,
            Pattern::Expression(expression) => match expression.regex() {
                Some(regex) => regex.is_match(record.full_line),
                None => expression
                    .evaluate(functions, variables, record)
                    .coercion_to_boolean(),
            },
            Pattern::Begin => false,
            Pattern::End => false,
        }
    }
}

pub(crate) fn parse_item_pattern(input: &str) -> IResult<&str, Pattern> {
    let parse_pattern = alt((
        map(tag("BEGIN"), |_| Pattern::Begin),
        map(tag("END"), |_| Pattern::End),
        map(parse_expression, |expr| Pattern::Expression(expr)),
    ));
    map(opt(parse_pattern), |pattern_opt| {
        pattern_opt.unwrap_or(Pattern::MatchEverything)
    })(input)
}
