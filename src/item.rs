use crate::basic_types;
use crate::expression::Expression;

static EMPTY_STRING: &str = "";

pub enum Statement {
    Print(Expression),
}

pub struct Action {
    pub statements: Vec<Statement>,
}

impl Action {
    pub fn output_for_line<'a>(
        &self,
        context: &basic_types::Context,
        record: &basic_types::Record<'a>,
    ) -> Vec<String> {
        self.statements
            .iter()
            .map(|statement| match statement {
                Statement::Print(expression) => expression.evaluate(context).coerce_to_string(),
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
