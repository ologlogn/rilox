use std::rc::Rc;
use crate::lexer::token::{Token};
use crate::parser::expr::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionType {
    NONE,
    FUNCTION,
    METHOD,
    INIT,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClassType {
    NONE,
    CLASS,
}
#[derive(Debug, Clone)]
pub enum Statement {
    ExpressionStmt(Expr),
    PrintStmt(Expr),
    VarStmt(Token, Option<Expr>), // name, optional initializer
    BlockStmt(Vec<Statement>),
    IfStmt(Expr, Box<Statement>, Option<Box<Statement>>),
    WhileStmt(Expr, Box<Statement>),
    FunctionStmt(Token, Vec<Token>, Rc<Box<Statement>>, FunctionType),
    ReturnStmt(Token, Option<Expr>),
    ClassStmt(Token, Vec<Statement>), // name, methods
}
