use regex::Regex;

pub enum UnaryExpression {
}

pub enum NonunaryExpression {
    ExtendedRegex(Regex)
}

pub enum Expression {
    Unary(UnaryExpression),
    Nonunary(NonunaryExpression),
}
