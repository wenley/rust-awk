use crate::basic_types;

static EMPTY_STRING: &str = "";

pub enum Statement {
    Print(basic_types::Field),
}

pub struct Action {
    pub statements: Vec<Statement>,
}

impl Action {
    pub fn output_for_line<'a>(&self, record: &basic_types::Record<'a>) -> Vec<&'a str> {
        self.statements
            .iter()
            .map(|statement| match statement {
                Statement::Print(basic_types::Field::WholeLine) => record.full_line,
                Statement::Print(basic_types::Field::Indexed(index)) => {
                    record.fields.get(index - 1).unwrap_or(&EMPTY_STRING)
                }
            })
            .collect()
    }
}

pub enum Pattern {
    MatchEverything,
    Begin,
    End,
}

impl Pattern {
    pub fn matches<'a>(&self, _record: &basic_types::Record<'a>) -> bool {
        match self {
            Pattern::MatchEverything => true,
            Pattern::Begin => false,
            Pattern::End => false,
        }
    }
}

pub struct Item {
    pub pattern: Pattern,
    pub action: Action,
}
