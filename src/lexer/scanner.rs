use crate::error::error;
use crate::lexer::token::{Literal, Token, TokenType};
pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}
fn is_alpha(c: char) -> bool {
    (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_'
}
fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}
impl Scanner {
    pub fn new(input: String) -> Scanner {
        Scanner {
            source: input.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()
        }
        self.tokens
            .push(Token::new(TokenType::EOF, String::new(), None, self.line));
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let ch: char = self.advance();
        match ch {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                if self.if_next_char_then_advance('=') {
                    self.add_token(TokenType::BangEqual, None)
                } else {
                    self.add_token(TokenType::Bang, None)
                }
            }
            '=' => {
                if self.if_next_char_then_advance('=') {
                    self.add_token(TokenType::EqualEqual, None)
                } else {
                    self.add_token(TokenType::Equal, None)
                }
            }
            '>' => {
                if self.if_next_char_then_advance('=') {
                    self.add_token(TokenType::GreaterEqual, None)
                } else {
                    self.add_token(TokenType::Greater, None)
                }
            }
            '<' => {
                if self.if_next_char_then_advance('=') {
                    self.add_token(TokenType::LessEqual, None)
                } else {
                    self.add_token(TokenType::Less, None)
                }
            }
            '/' => {
                if self.if_next_char_then_advance('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance(); // ignore comments
                    }
                } else {
                    self.add_token(TokenType::Slash, None)
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => {
                while self.peek() != '"' && !self.is_at_end() {
                    if self.peek() == '\n' {
                        self.line += 1; // multi line string is supported.
                    }
                    self.advance();
                }
                if self.is_at_end() {
                    error(self.line, "Unterminated string".to_string());
                } else {
                    self.advance();
                    let val = self.source[self.start + 1..self.current - 1].to_vec();
                    self.add_token(
                        TokenType::String,
                        Some(Literal::String(val.iter().collect())),
                    );
                }
            }
            _ => {
                if is_digit(ch) {
                    while is_digit(self.peek()) {
                        self.advance();
                    }
                    if self.peek() == '.' && is_digit(self.peek_next()) {
                        self.advance();
                        while is_digit(self.peek()) {
                            self.advance();
                        }
                    }
                    let val: f64 = self.source[self.start..self.current]
                        .to_vec()
                        .iter()
                        .collect::<String>()
                        .parse()
                        .unwrap();
                    self.add_token(TokenType::Number, Some(Literal::Number(val)));
                } else if is_alpha(ch) {
                    while is_alphanumeric(self.peek()) {
                        self.advance();
                    }
                    let val = self.source[self.start..self.current]
                        .iter()
                        .collect::<String>();
                    if let Some(token_type) = crate::lexer::token::keywords(val) {
                        self.add_token(token_type, None)
                    } else {
                        self.add_token(TokenType::Identifier, None)
                    }
                } else {
                    error(self.line, format!("Unexpected character {} ", ch))
                }
            }
        }
    }
    fn if_next_char_then_advance(&mut self, expected: char) -> bool {
        if self.peek() != expected {
            return false;
        }
        self.advance();
        true
    }
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }
    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        self.tokens.push(Token::new(
            token_type,
            self.source[self.start..self.current]
                .to_vec()
                .iter()
                .collect(),
            literal,
            self.line,
        ))
    }
}
