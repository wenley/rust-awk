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
    printable::Printable,
};

pub(crate) enum Pattern {
    MatchEverything,
    Expression(Box<dyn Expression>),
    Begin,
    End,
}

impl Pattern {
    pub(crate) fn matches(
        &self,
        functions: &Functions,
        context: &mut MutableContext,
    ) -> Printable<bool> {
        match self {
            Pattern::MatchEverything => Printable::wrap(true),
            Pattern::Expression(expression) => match expression.regex() {
                Some(regex) => {
                    Printable::wrap(regex.is_match(&context.fetch_field(0).coerce_to_string()))
                }
                None => Printable::wrap(
                    expression
                        .evaluate(functions, context)
                        .coercion_to_boolean(),
                ),
            },
            Pattern::Begin => Printable::wrap(false),
            Pattern::End => Printable::wrap(false),
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
