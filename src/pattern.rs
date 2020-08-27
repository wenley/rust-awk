use crate::basic_types::Record;
use crate::expression::Expression;

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
