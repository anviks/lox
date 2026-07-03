use std::fmt::{self, Write};

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
        f.pad(&pascal_to_upper_snake(format!("{:?}", self)))
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

fn show(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\x{:x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let tail = 6.min(max / 3);
    let head = max.saturating_sub(tail + 3);
    let head_s: String = s.chars().take(head).collect();
    let tail_s: String = s.chars().skip(count - tail).collect();
    format!("{head_s}…{tail_s}")
}

const MAX_CELL: usize = 32;

pub(crate) fn dump_tokens(tokens: &[Token]) -> String {
    let rows: Vec<[String; 4]> = tokens
        .iter()
        .map(|t| {
            [
                format!("{}", t.token_type),
                truncate(&show(&t.lexeme), MAX_CELL),
                match &t.literal {
                    Some(s) => truncate(&show(s), MAX_CELL),
                    None => "-".to_string(),
                },
                format!(
                    "{}:{} - {}:{}",
                    t.span.line_start, t.span.col_start, t.span.line_end, t.span.col_end
                ),
            ]
        })
        .collect();

    let headers = ["TYPE", "LEXEME", "LITERAL", "SPAN"];
    let mut widths = headers.map(|h| h.len());
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }

    let mut out = String::new();
    let mut push_row = |cells: &[&str]| {
        for (i, cell) in cells.iter().enumerate() {
            let _ = write!(out, "{:<width$}  ", cell, width = widths[i]);
        }
        while out.ends_with(' ') {
            out.pop();
        }
        out.push('\n');
    };

    push_row(&headers);
    for row in &rows {
        let cells = row.each_ref().map(String::as_str);
        push_row(&cells);
    }
    out
}
