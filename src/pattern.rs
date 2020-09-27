use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    IResult,
};

use crate::{
    basic_types::MutableContext,
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
    pub(crate) fn matches(&self, functions: &Functions, context: &mut MutableContext) -> bool {
        match self {
            Pattern::MatchEverything => true,
            Pattern::Expression(expression) => match expression.regex() {
                Some(regex) => regex.is_match(context.record.unwrap().full_line),
                None => expression
                    .evaluate(functions, context)
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
