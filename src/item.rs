use crate::basic_types;
use crate::expression::Expression;

static EMPTY_STRING: &str = "";

pub enum Statement {
    IfElse {
        condition: Expression,
        if_branch: Box<Statement>,
        else_branch: Box<Statement>,
    },
    Print(Expression),
    Assign {
        variable_name: String,
        value: Expression,
    },
}

impl Statement {
    pub fn evaluate(&self, context: &mut basic_types::Context) -> String {
        match self {
            Statement::Print(expression) => expression.evaluate(context).coerce_to_string(),
            Statement::IfElse {
                condition,
                if_branch,
                else_branch,
            } => {
                let result = condition.evaluate(context).coercion_to_boolean();
                if result {
                    if_branch.evaluate(context)
                } else {
                    else_branch.evaluate(context)
                }
            }
            Statement::Assign {
                variable_name,
                value,
            } => {
                let value = value.evaluate(context);
                context.assign_variable(&variable_name, value);
                EMPTY_STRING.to_string()
            }
        }
    }
}

pub struct Action {
    pub statements: Vec<Statement>,
}

impl Action {
    pub fn output_for_line<'a>(
        &self,
        context: &mut basic_types::Context,
        record: &basic_types::Record<'a>,
    ) -> Vec<String> {
        self.statements
            .iter()
            .map(|statement| statement.evaluate(context))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_statement_produces_value() {
        let mut empty_context = basic_types::Context::empty();
        let print_statement = Statement::Print(Expression::StringLiteral("hello".to_string()));
        assert_eq!(print_statement.evaluate(&mut empty_context), "hello",);
    }

    #[test]
    fn if_produces_correct_value() {
        let mut empty_context = basic_types::Context::empty();

        let if_conditional = Statement::IfElse {
            condition: Expression::StringLiteral("not empty".to_string()),
            if_branch: Box::new(Statement::Print(Expression::StringLiteral(
                "if-branch".to_string(),
            ))),
            else_branch: Box::new(Statement::Print(Expression::StringLiteral(
                "else".to_string(),
            ))),
        };
        assert_eq!(if_conditional.evaluate(&mut empty_context), "if-branch",);

        let else_conditional = Statement::IfElse {
            condition: Expression::StringLiteral("".to_string()),
            if_branch: Box::new(Statement::Print(Expression::StringLiteral(
                "if-branch".to_string(),
            ))),
            else_branch: Box::new(Statement::Print(Expression::StringLiteral(
                "else".to_string(),
            ))),
        };
        assert_eq!(else_conditional.evaluate(&mut empty_context), "else",);
    }

    #[test]
    fn assignment_updates_context() {
        let mut context = basic_types::Context::empty();

        let assign = Statement::Assign {
            variable_name: "foo".to_string(),
            value: Expression::AddBinary {
                left: Box::new(Expression::NumericLiteral(
                    basic_types::NumericValue::Integer(1),
                )),
                right: Box::new(Expression::NumericLiteral(
                    basic_types::NumericValue::Integer(2),
                )),
            },
        };
        assign.evaluate(&mut context);
        assert_eq!(
            context.fetch_variable("foo"),
            basic_types::Value::Numeric(basic_types::NumericValue::Integer(3)),
        );
    }
}
