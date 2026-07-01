use std::fmt;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Star,
    Dot,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Slash,

    Equal,
    EqualEqual,
    Bang,
    BangEqual,

    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    String,
    Number,

    Identifier,

    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

fn pascal_to_upper_snake(s: String) -> String {
    let mut result = String::new();
    for (i, ch) in s.char_indices() {
        if ch.is_uppercase() && i != 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_uppercase());
    }
    result
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", pascal_to_upper_snake(format!("{:?}", self)))
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) struct Span {
    pub(crate) line_start: u32,
    pub(crate) col_start: u32,
    pub(crate) line_end: u32,
    pub(crate) col_end: u32,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) struct Token {
    pub(crate) token_type: TokenType,
    pub(crate) lexeme: String,
    pub(crate) literal: Option<String>,
    pub(crate) span: Span,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.token_type,
            self.lexeme,
            self.literal.as_deref().unwrap_or("null")
        )
    }
}
