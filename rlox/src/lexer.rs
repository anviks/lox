use crate::{
    helpers::format_float,
    token::{Span, Token, TokenType},
};

fn get_token_type_for_identifier(identifier: &str) -> TokenType {
    match identifier {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fun" => TokenType::Fun,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        _ => TokenType::Identifier,
    }
}

pub struct Lexer {
    pub source: Vec<char>,
    pub current: usize,
    pub line: u32,
    pub column: u32,
    tok_start_line: u32,
    tok_start_col: u32,
    pub encountered_error: bool,
}

impl Lexer {
    pub fn new(source: &String) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            tok_start_line: 1,
            tok_start_col: 1,
            encountered_error: false,
        }
    }

    fn eof(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek_at(&self, offset: usize) -> char {
        if self.current + offset >= self.source.len() {
            return '\0';
        }
        self.source[self.current + offset]
    }

    fn peek(&self) -> char {
        self.peek_at(0)
    }

    fn consume(&mut self) -> char {
        let c = self.peek();
        self.current += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn string(&mut self) -> Option<Token> {
        let mut s = String::new();
        self.consume();
        while !self.eof() && self.peek() != '"' {
            s.push(self.consume());
        }
        if self.eof() {
            eprintln!("[line {}] Error: Unterminated string.", self.line);
            self.encountered_error = true;
            None
        } else {
            self.consume();
            Some(self.make_token_with_literal(TokenType::String, format!("\"{}\"", s), s))
        }
    }

    fn number(&mut self) -> Option<Token> {
        let mut num_str = String::new();
        while !self.eof() && (self.peek().is_ascii_digit() || self.peek() == '.') {
            num_str.push(self.consume());
        }

        let parsed_num = num_str.parse::<f64>();

        match parsed_num {
            Ok(num) => {
                Some(self.make_token_with_literal(TokenType::Number, num_str, format_float(num)))
            }
            Err(_) => {
                eprintln!(
                    "[line {}] Error: Invalid number literal: {}",
                    self.line, num_str
                );
                self.encountered_error = true;
                None
            }
        }
    }

    fn identifier(&mut self) -> Token {
        let mut identifier = self.consume().to_string();
        while !self.eof() {
            let c = self.peek();
            if !c.is_ascii_alphanumeric() && c != '_' {
                break;
            }
            identifier.push(self.consume());
        }

        self.make_token(
            get_token_type_for_identifier(identifier.as_str()),
            identifier,
        )
    }

    fn block_comment(&mut self) {
        let mut depth = 0;
        while !self.eof() {
            if self.peek() == '/' && self.peek_at(1) == '*' {
                self.consume();
                depth += 1;
            } else if self.peek() == '*' && self.peek_at(1) == '/' {
                self.consume();
                depth -= 1;
            }

            self.consume();

            if depth == 0 {
                break;
            }
        }
    }

    fn current_span(&self) -> Span {
        Span {
            line_start: self.tok_start_line,
            col_start: self.tok_start_col,
            line_end: self.line,
            col_end: self.column,
        }
    }

    fn make_token(&self, token_type: TokenType, lexeme: String) -> Token {
        Token {
            token_type,
            lexeme,
            literal: None,
            span: self.current_span(),
        }
    }

    fn make_token_with_literal(
        &self,
        token_type: TokenType,
        lexeme: String,
        literal: String,
    ) -> Token {
        Token {
            token_type,
            lexeme,
            literal: Some(literal),
            span: self.current_span(),
        }
    }

    pub fn analyze(&mut self) -> Vec<Token> {
        self.encountered_error = false;

        let mut tokens: Vec<Token> = vec![];

        while !self.eof() {
            self.tok_start_line = self.line;
            self.tok_start_col = self.column;

            let double_char = match (self.peek(), self.peek_at(1)) {
                ('=', '=') => Some(TokenType::EqualEqual),
                ('!', '=') => Some(TokenType::BangEqual),
                ('<', '=') => Some(TokenType::LessEqual),
                ('>', '=') => Some(TokenType::GreaterEqual),
                _ => None,
            };

            let single_char = match self.peek() {
                '(' => Some(TokenType::LeftParen),
                ')' => Some(TokenType::RightParen),
                '{' => Some(TokenType::LeftBrace),
                '}' => Some(TokenType::RightBrace),
                '*' => Some(TokenType::Star),
                '.' => Some(TokenType::Dot),
                ',' => Some(TokenType::Comma),
                '+' => Some(TokenType::Plus),
                '-' => Some(TokenType::Minus),
                ';' => Some(TokenType::Semicolon),
                '=' => Some(TokenType::Equal),
                '!' => Some(TokenType::Bang),
                '<' => Some(TokenType::Less),
                '>' => Some(TokenType::Greater),
                _ => None,
            };

            if let Some(token_type) = double_char {
                let lexeme = self.consume().to_string() + &self.consume().to_string();
                tokens.push(self.make_token(token_type, lexeme));
            } else if let Some(token_type) = single_char {
                let lexeme = self.consume().to_string();
                tokens.push(self.make_token(token_type, lexeme));
            } else {
                match self.peek() {
                    '"' => tokens.extend(self.string()),
                    '0'..='9' => tokens.extend(self.number()),
                    'a'..='z' | 'A'..='Z' | '_' => tokens.push(self.identifier()),
                    ' ' | '\t' => {
                        self.consume();
                    }
                    '\n' => {
                        self.consume();
                    }
                    '/' if self.peek_at(1) == '/' => {
                        while !self.eof() && self.peek() != '\n' {
                            self.consume();
                        }
                    }
                    '/' if self.peek_at(1) == '*' => self.block_comment(),
                    '/' => {
                        let lexeme = self.consume().to_string();
                        tokens.push(self.make_token(TokenType::Slash, lexeme))
                    }
                    _ => {
                        let c = self.consume();
                        eprintln!("[line {}] Error: Unexpected character: {}", self.line, c);
                        self.encountered_error = true;
                    }
                }
            }
        }

        self.tok_start_line = self.line;
        self.tok_start_col = self.column;
        tokens.push(self.make_token(TokenType::Eof, String::new()));

        self.current = 0;
        self.line = 1;
        self.column = 1;

        tokens
    }
}
