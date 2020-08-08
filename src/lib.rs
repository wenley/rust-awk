
enum Field {
    WholeLine,
    Indexed(u8),
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
                command: Command::Print(Field::WholeLine),
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

impl ProgramRun<'_> {
    pub fn output_for_line<'a>(&self, line: &'a str) -> Vec<&'a str> {
        self.program
            .rules
            .iter()
            .flat_map(|rule| {
                match rule.command {
                    Command::Print(Field::WholeLine) => {
                        vec![line]
                    }
                    Command::Print(Field::Indexed(u8)) => {
                        panic!("indexed print")
                    }
                }
            })
            .collect()
    }
}
