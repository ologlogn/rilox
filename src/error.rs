use crate::error::Error::RuntimeError;
use crate::interpreter::value::Value;
use crate::lexer::token::Token;
use std::fmt::{Display, Formatter};

//Prints line with error message
pub fn error(line: usize, message: String) {
    eprintln!("line {}, {}", line, message);
}
#[derive(Debug)]
pub enum Error {
    RuntimeError(String),
    Return(Value),
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError(msg) => write!(f, "{}", msg),
            Error::Return(val) => write!(f, "{}", val),
        }
    }
}

pub fn runtime_error(token: Token, message: &str) {
    eprintln!(
        "[line {}] Runtime error {}: {}",
        token.line, token.lexeme, message
    );
    std::process::exit(70);
}
