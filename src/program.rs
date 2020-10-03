use nom::{
    branch::alt,
    character::complete::multispace0,
    combinator::{all_consuming, map},
    multi::many1,
    sequence::delimited,
    IResult,
};
use std::collections::HashMap;

use crate::{
    function::{parse_function, FunctionDefinition, Functions},
    item::{parse_item, Item},
};

pub struct Program {
    pub(crate) items: Vec<Item>,
    pub(crate) functions: Functions,
}

enum ParsedThing {
    Item(Item),
    Function(FunctionDefinition),
}

fn parse_item_list(input: &str) -> IResult<&str, (Vec<Item>, Vec<FunctionDefinition>)> {
    let parse_thing = alt((
        map(parse_item, |item| ParsedThing::Item(item)),
        map(parse_function, |function| ParsedThing::Function(function)),
    ));

    map(
        many1(delimited(multispace0, parse_thing, multispace0)),
        |things| {
            let mut items = vec![];
            let mut functions = vec![];

            for thing in things {
                match thing {
                    ParsedThing::Item(item) => {
                        items.push(item);
                    }
                    ParsedThing::Function(function) => {
                        functions.push(function);
                    }
                }
            }

            (items, functions)
        },
    )(input)
}

pub fn parse_program(program_text: &str) -> Program {
    match all_consuming(parse_item_list)(program_text) {
        Ok((_, (items, functions))) => {
            let mut function_map = HashMap::new();
            for func in functions {
                function_map.insert(func.name.clone(), func);
            }
            Program {
                items: items,
                functions: function_map,
            }
        }
        Err(e) => panic!("Could not parse! {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program() {
        // Assert no panic
        parse_program(
            r#"{ print(1);
            print(2.0);
            print("hello");
        }"#,
        );
    }

    #[test]
    fn test_parse_program_with_function() {
        // Assert no panic
        let program = parse_program(
            r#"{ print(1);
            print(2.0);
            print("hello");
        }
        function foo(a) {
          print("hello");
        }
        "#,
        );
        assert_eq!(program.items.len(), 1);
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_bad_program() {
        // Assert no panic
        let program = parse_program(
            r#"function store(val) {
  a = val;
}
{
  print($0, a);
  b = store($0);
}"#,
        );
        assert_eq!(program.items.len(), 1);
        assert_eq!(program.functions.len(), 1);
    }
}
