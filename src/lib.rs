pub mod expression;
pub mod basic_types;

enum Action {
    Print(basic_types::Field),
}

impl Action {
    pub fn output_for_line<'a>(&self, record: &Record<'a>) -> Vec<&'a str> {
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
    pub fn matches<'a>(&self, _record: &Record<'a>) -> bool {
        match self {
            Pattern::MatchEverything => { true }
            Pattern::Begin => { false }
            Pattern::End => { false }
            Pattern::Expression(_compare) => { false }
        }
    }
}

pub struct Rule {
    pattern: Pattern,
    action: Action,
}

pub struct Program {
    rules: Vec<Rule>,
}

pub fn parse_program(_program_text: String) -> Program {
    Program {
        rules: vec![
            Rule {
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

pub struct Record<'a> {
    pub full_line: &'a str,
    pub fields: &'a Vec<&'a str>,
}

static EMPTY_STRING: &str = "";

impl ProgramRun<'_> {
    pub fn output_for_line<'a>(&self, record: &Record<'a>) -> Vec<&'a str> {
        self.program
            .rules
            .iter()
            .filter(|rule| {
                rule.pattern.matches(record)
            })
            .flat_map(|rule| {
                rule.action.output_for_line(record)
            })
            .collect()
    }

    pub fn execute_begin(&mut self) {
        self.program.rules.iter()
            .filter(|rule| {
                match rule.pattern {
                    Pattern::Begin => { true }
                    _ => { false }
                }
            })
            .for_each(|begin_rule| {
                begin_rule.action.execute(self)
            });
    }
}
