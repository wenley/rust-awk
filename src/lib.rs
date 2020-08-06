
struct Pattern;

impl Pattern {
    pub fn matches(record: String) -> bool {
        true
    }
}

struct Command;

impl Command {
    pub fn execute(record: String) {}
}

struct Rule {
    pattern: Pattern,
    command: Command,
}

struct Program {
    rules: Vec<Rule>,
}

pub trait Runnable {
}

impl Runnable for Program {
}

pub fn parse_program(program_text: String) -> Box<dyn Runnable> {
    Box::new(Program { rules: vec![] })
}
