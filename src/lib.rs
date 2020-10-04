extern crate nom;
extern crate regex;

mod action;
mod basic_types;
mod expression;
mod function;
mod item;
mod parse_args;
mod pattern;
mod printable;
mod program;
mod program_run;
mod value;

use crate::program::parse_program;

pub use program_run::ProgramRun;

pub fn start_run(args: Vec<String>) -> (ProgramRun, Vec<String>) {
    let (program_string, parsed_args) = parse_args::parse_args(args);
    let program = parse_program(&program_string);

    let mut run = ProgramRun::new_for_program(program);

    run.apply_args(&parsed_args);

    (run, parsed_args.filepaths_to_parse)
}

#[cfg(test)]
mod test_utilities {
    use super::*;
    use std::collections::HashMap;

    pub(crate) fn empty_functions_and_variables() -> (function::Functions, basic_types::Variables) {
        (HashMap::new(), basic_types::Variables::empty())
    }
}
