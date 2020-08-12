pub mod expression;
pub mod basic_types;

enum Action {
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

    pub fn execute(&self, _run: &mut ProgramRun) {
    }
}

enum Pattern {
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
    pattern: Pattern,
    action: Action,
}

pub struct Program {
    items: Vec<Item>,
}

pub fn parse_program(_program_text: String) -> Program {
    Program {
        items: vec![
            Item {
                pattern: Pattern::MatchEverything,
                action: Action::Print(basic_types::Field::Indexed(3)),
            }
        ],
    }
}

pub struct Context {
}

pub struct ProgramRun<'a> {
    program: &'a Program,
    context: Context,
}

pub fn start_run<'a>(program: &'a Program) -> ProgramRun<'a> {
    ProgramRun {
        program: program,
        context: Context {},
    }
}

static EMPTY_STRING: &str = "";

impl ProgramRun<'_> {
    pub fn output_for_line<'a>(&self, record: &basic_types::Record<'a>) -> Vec<&'a str> {
        self.program
            .items
            .iter()
            .filter(|item| {
                item.pattern.matches(record)
            })
            .flat_map(|item| {
                item.action.output_for_line(record)
            })
            .collect()
    }

    pub fn execute_begin(&mut self) {
        self.program.items.iter()
            .filter(|item| {
                match item.pattern {
                    Pattern::Begin => { true }
                    _ => { false }
                }
            })
            .for_each(|begin_rule| {
                begin_rule.action.execute(self)
            });
    }
}
