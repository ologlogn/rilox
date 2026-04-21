use crate::interpreter::value::Value;
use crate::lexer::token::Token;

//Prints line with error message
pub fn error(line: usize, message: String) {
    eprintln!("line {}, {}", line, message);
}
#[derive(Debug)]
pub enum Error {
    RuntimeError(String),
    Return(Value),
}

pub fn runtime_error(token: Token, message: &str) {
    eprintln!(
        "[line {}] Runtime error {}: {}",
        token.line, token.lexeme, message
    );
    std::process::exit(70);
}
