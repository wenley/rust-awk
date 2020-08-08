
enum Field {
    WholeLine,
    Indexed(usize),
}

enum Command {
    Print(Field),
}

pub struct Rule {
    command: Command,
}

pub struct Program {
    rules: Vec<Rule>,
}

pub fn parse_program(_program_text: String) -> Program {
    Program {
        rules: vec![
            Rule {
                command: Command::Print(Field::Indexed(1)),
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
            .map(|rule| {
                match rule.command {
                    Command::Print(Field::WholeLine) => {
                        record.full_line
                    }
                    Command::Print(Field::Indexed(index)) => {
                        record.fields.get(index - 1).unwrap_or(&empty_string)
                    }
                }
            })
            .collect()
    }
}
