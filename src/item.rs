use crate::expression;
use crate::basic_types;

static EMPTY_STRING: &str = "";

pub enum Action {
    Print(basic_types::Field),
}

impl Action {
    pub fn output_for_line<'a>(&self, record: &basic_types::Record<'a>) -> Vec<&'a str> {
        match self {
            Action::Print(basic_types::Field::WholeLine) => {
                vec![record.full_line]
            }
            Action::Print(basic_types::Field::Indexed(index)) => {
                vec![record.fields.get(index - 1).unwrap_or(&EMPTY_STRING)]
            }
        }
    }
}

pub enum Pattern {
    MatchEverything,
    Begin,
    End,
    Expression(expression::Expression)
}

impl Pattern {
    pub fn matches<'a>(&self, _record: &basic_types::Record<'a>) -> bool {
        match self {
            Pattern::MatchEverything => { true }
            Pattern::Begin => { false }
            Pattern::End => { false }
            Pattern::Expression(_compare) => { false }
        }
    }
}

pub struct Item {
    pub pattern: Pattern,
    pub action: Action,
}

