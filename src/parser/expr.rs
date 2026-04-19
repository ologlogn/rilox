use crate::lexer::token::Literal;
use crate::lexer::token::{Token};
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Variable(Token),
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        token: Token, // for runtime errors
        args: Vec<Expr>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Unary { operator, right } => {
                write!(f, "({}{})", operator.lexeme, right.print())
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                write!(
                    f,
                    "({} {} {})",
                    operator.lexeme,
                    left.print(),
                    right.print()
                )
            }
            Expr::Grouping { expr } => {
                write!(f, "(group {})", expr.print())
            }
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Assignment { name, value } => {
                write!(f, "({} = {}", name, value.print())
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                write!(
                    f,
                    "({} {} {})",
                    operator.lexeme,
                    left.print(),
                    right.print()
                )
            }
            Expr::Call {
                callee,
                token,
                args,
            } => {
                write!(
                    f,
                    "({} {} {})",
                    callee.print(),
                    token.lexeme,
                    args.iter()
                        .map(|arg| arg.print())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }
}

impl Expr {
    pub fn print(&self) -> String {
        match self {
            Expr::Literal(lit) => format!("{}", lit),
            Expr::Unary { operator, right } => {
                format!("({}{})", operator.lexeme, right.print())
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                format!("({} {} {})", operator.lexeme, left.print(), right.print())
            }
            Expr::Grouping { expr } => {
                format!("(group {})", expr.print())
            }
            Expr::Variable(name) => format!("${}", name),
            Expr::Assignment { name, value } => {
                format!("({} = {})", name, value.print())
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                format!("({} {} {})", operator.lexeme, left.print(), right.print())
            }
            Expr::Call {
                callee,
                token,
                args,
            } => {
                format!(
                    "{},{},{}",
                    callee.print(),
                    token.lexeme,
                    args.iter()
                        .map(|arg| arg.print())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }
}
