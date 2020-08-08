
enum Field {
    WholeLine,
    Indexed(usize),
}

enum Command {
    Print(Field),
}

impl Command {
    pub fn output_for_line<'a>(&self, record: &Record<'a>) -> Vec<&'a str> {
        match self {
            Command::Print(Field::WholeLine) => {
                vec![record.full_line]
            }
            Command::Print(Field::Indexed(index)) => {
                vec![record.fields.get(index - 1).unwrap_or(&empty_string)]
            }
        }
    }
}

pub struct Compare {
}

enum Pattern {
    MatchEverything,
    Compare(Compare),
}

impl Pattern {
    pub fn matches<'a>(&self, _record: &Record<'a>) -> bool {
        match self {
            Pattern::MatchEverything => { true }
            Pattern::Compare(_compare) => { false }
        }
    }
}

pub struct Rule {
    pattern: Pattern,
    command: Command,
}

pub struct Program {
    rules: Vec<Rule>,
}

pub fn parse_program(_program_text: String) -> Program {
    Program {
        rules: vec![
            Rule {
                pattern: Pattern::MatchEverything,
                command: Command::Print(Field::Indexed(3)),
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

static empty_string: &str = "";

impl ProgramRun<'_> {
    pub fn output_for_line<'a>(&self, record: &Record<'a>) -> Vec<&'a str> {
        self.program
            .rules
            .iter()
            .filter(|rule| {
                rule.pattern.matches(record)
            })
            .flat_map(|rule| {
                rule.command.output_for_line(record)
            })
            .collect()
    }
}
