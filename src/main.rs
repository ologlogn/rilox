mod common;
mod error;
mod interpreter;
mod lexer;
mod parser;

use crate::interpreter::interpreter::Interpreter;
use crate::lexer::scanner::Scanner;
use crate::parser::parser::Parser;
use std::io::Write;
use std::process::exit;
use std::{env, fs, io};
use crate::interpreter::resolver::Resolver;
/*
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: rilox [script]");
        exit(-1);
    } else if args.len() == 2 {
        run_file(args[1].clone());
    } else {
        run_prompt();
    }
}

fn main() {
    let expr = Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 1),
            right: Box::new(Literal(lexer::token::Literal::Number(10.123))),
        }),
        operator: Token::new(TokenType::Star, "*".to_string(), None, 1),
        right: Box::new(Expr::Grouping {
            expr: Box::new(Literal(lexer::token::Literal::Number(45.67))),
        }),
    };

    println!("{}", expr.print());
}

 */
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut interpreter = Interpreter::new();
    let mut resolver = Resolver::new();
    if args.len() > 2 {
        println!("Usage: rilox [script]");
        exit(-1);
    } else if args.len() == 2 {
        run_file(args[1].clone(), &mut interpreter, &mut resolver);
    } else {
        run_prompt(&mut interpreter, &mut resolver);
    }
}
pub fn interpret(source: String, interpreter: &mut Interpreter, resolver: &mut Resolver) -> i32 {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(stmts) => stmts,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            return 65;
        }
    };
    for statement in &statements {
        resolver.resolve_statement(statement);
    }
    interpreter.locals = resolver.locals.clone();
    for stmt in &statements {
        if interpreter.had_error {
            break;
        }
        interpreter.execute_stmt(stmt).unwrap();
    }
    if interpreter.had_error { 70 } else { 0 }
}
fn run_prompt(interpreter: &mut Interpreter, resolver: &mut Resolver) {
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        let bytes_read = stdin.read_line(&mut line).unwrap();
        if bytes_read == 0 {
            break;
        }
        if line.is_empty() {
            continue;
        }
        interpret(line, interpreter, resolver);
    }
}

fn run_file(file_name: String, interpreter: &mut Interpreter, resolver: &mut Resolver) {
    let content: String = fs::read_to_string(&file_name).unwrap();
    interpret(content, interpreter, resolver);
}
