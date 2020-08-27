use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    IResult,
};

use crate::{
    basic_types::Record,
    expression::{parse_expression, Expression},
};

pub(crate) enum Pattern {
    MatchEverything,
    Expression(Expression),
    Begin,
    End,
}

impl Pattern {
    pub fn matches<'a>(&self, _record: &Record<'a>) -> bool {
        match self {
            Pattern::MatchEverything => true,
            // TODO: Make this proper
            Pattern::Expression(_) => true,
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
