use crate::lexer::token::{Token};
use crate::parser::expr::Expr;

#[derive(Debug, Clone)]
pub enum Statement {
    ExpressionStmt(Expr),
    PrintStmt(Expr),
    VarStmt(String, Option<Expr>), // name, optional initializer
    BlockStmt(Vec<Statement>),
    IfStmt(Expr, Box<Statement>, Option<Box<Statement>>),
    WhileStmt(Expr, Box<Statement>),
    FunctionStmt(Token, Vec<Token>, Box<Statement>), //
    ReturnStmt(Token, Option<Expr>),
}
