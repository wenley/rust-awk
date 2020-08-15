use regex::Regex;

pub enum Literal {
    StringLiteral(String),
    IntegerLiteral(u64),
    FloatLiteral(f64),

}

#[cfg(test)]
mod tests {
    #[test]
    fn literals_can_evaluate() {
        assert_eq!(Literal::StringLiteral("hello").evaluate(), "hello");
    }
}
