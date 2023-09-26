
use crate::string_builder;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ParenthesisSide {
    Left = 0,
    Right = 1,
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    Name(String),
    ArrayBracket(ParenthesisSide),
    Literal(i32),
    Comment(String),
}

#[derive(derive_new::new, Default, Debug)]
pub struct TokenLine {
    pub tokens: Vec<Token>,
    pub line: String,
}

pub fn format_token_collection(col: &[TokenLine]) -> String {
    return string_builder::reduce_additive(col.iter(), |e| format!("\"{}\" {:?}\n", e.line, e.tokens));
}
