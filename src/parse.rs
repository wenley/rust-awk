extern crate nom;
extern crate regex;

use crate::{
    expression::Expression,
    item::{parse_item_list, Action, Item},
    pattern::Pattern,
    statement::Statement,
};

pub struct Program {
    pub items: Vec<Item>,
}

/* - - - - - - - - - -
 * Statement Parsers
 * - - - - - - - - - - */

pub fn parse_program(program_text: &str) -> Program {
    let default_program = Program {
        items: vec![Item {
            pattern: Pattern::MatchEverything,
            action: Action {
                statements: vec![Statement::Print(Expression::StringLiteral(
                    "hi".to_string(),
                ))],
            },
        }],
    };

    match parse_item_list(program_text) {
        Ok((_, items)) => Program { items: items },
        _ => default_program,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::NumericValue;

    #[test]
    fn test_parse_program() {
        let program = parse_program(
            r#"{ print(1);
            print(2.0);
            print("hello");
        }"#,
        );

        assert_eq!(program.items[0].action.statements.len(), 3);
    }
}
