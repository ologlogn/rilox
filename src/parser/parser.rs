use crate::lexer::token::{Literal, Token, TokenType};
use crate::parser::expr::Expr;
use crate::parser::stmt::{FunctionType, Statement};
use std::rc::Rc;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }
    fn match_any(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    fn check(&mut self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == *token_type
        }
    }
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let ex = self.or()?; // left side
        if self.match_any(&[TokenType::Equal]) {
            let equal = self.previous().clone();
            let value = self.assignment()?; // right side
            if let Expr::Variable(var_token) = ex {
                Ok(Expr::Assignment {
                    name: var_token.clone(),
                    value: Box::new(value),
                })
            } else if let Expr::Get { object, name } = ex {
                // if left side was Get, we change it to Set, the last property is getting reassigned.
                Ok(Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                })
            } else {
                Err(ParseError {
                    message: "Invalid assignment target".to_string(),
                    line: equal.line,
                })
            }
        } else {
            Ok(ex)
        }
    }
    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;
        while self.match_any(&[TokenType::Or]) {
            let or = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: or,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.match_any(&[TokenType::And]) {
            let and = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: and,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    // equality -> comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?; // pointer
        while self.match_any(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    // comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.match_any(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.match_any(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.match_any(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_any(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.primary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?; // function name ->
        loop {
            if self.match_any(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?
            } else if self.match_any(&[TokenType::Dot]) {
                let name = self.expect(&TokenType::Identifier)?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name: name.clone(),
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut args = Vec::new();
        if !self.check(&TokenType::RightParen) {
            args.push(self.expression()?);
            while self.match_any(&[TokenType::Comma]) {
                args.push(self.expression()?);
            }
        }
        self.expect(&TokenType::RightParen)?;
        Ok(Expr::Call {
            callee: Box::from(callee),
            args,
        })
    }
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_any(&[TokenType::False]) {
            Ok(Expr::Literal(Literal::Boolean(false)))
        } else if self.match_any(&[TokenType::True]) {
            Ok(Expr::Literal(Literal::Boolean(true)))
        } else if self.match_any(&[TokenType::Nil]) {
            Ok(Expr::Literal(Literal::Nil))
        } else if self.match_any(&[TokenType::Number, TokenType::String]) {
            Ok(Expr::Literal(self.previous().clone().literal.unwrap())) // might panic if scanner has bug
        } else if self.match_any(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.expect(&TokenType::RightParen)?;
            Ok(Expr::Grouping {
                expr: Box::new(expr),
            })
        } else if self.match_any(&[TokenType::This]) {
            Ok(Expr::This {
                token: self.previous().clone(),
            })
        } else if self.match_any(&[TokenType::Identifier]) {
            Ok(Expr::Variable(self.previous().clone()))
        } else if self.match_any(&[TokenType::Super]) {
            let keyword = self.previous().clone();
            self.expect(&TokenType::Dot)?;
            let method = self.expect(&TokenType::Identifier)?.clone();
            Ok(Expr::Super { keyword, method })
        } else {
            Err(ParseError {
                message: "Parse error".to_string(),
                line: self.peek().line,
            })
        }
    }
    fn expect(&mut self, typ: &TokenType) -> Result<&Token, ParseError> {
        if self.check(typ) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, got {:?}", typ, self.peek().token_type),
                line: self.peek().line,
            })
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_any(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_any(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_any(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_any(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_any(&[TokenType::LeftBrace]) {
            self.block_statement()
        } else if self.match_any(&[TokenType::Return]) {
            self.return_statement()
        } else {
            self.expression_statement()
        }
    }
    fn return_statement(&mut self) -> Result<Statement, ParseError> {
        let keyword = self.previous().clone();
        let mut expr = None;
        if !self.check(&TokenType::Semicolon) {
            expr = Some(self.expression()?);
        }
        self.expect(&TokenType::Semicolon)?;
        Ok(Statement::ReturnStmt(keyword, expr))
    }
    fn for_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenType::LeftParen)?;

        let initializer = if self.match_any(&[TokenType::Semicolon]) {
            None
        } else if self.match_any(&[TokenType::Var]) {
            Some(Box::from(self.var_declaration()?))
        } else {
            Some(Box::from(self.expression_statement()?))
        };

        let condition = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.expect(&TokenType::Semicolon)?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.expect(&TokenType::RightParen)?;

        let body = self.statement()?;

        // Desugar here instead of runtime
        let mut block_body = vec![body];
        if let Some(increment) = increment {
            block_body.push(Statement::ExpressionStmt(increment));
        }

        let loop_body = if let Some(condition) = condition {
            Statement::WhileStmt(condition, Box::new(Statement::BlockStmt(block_body)))
        } else {
            Statement::BlockStmt(block_body)
        };

        let mut statements = Vec::new();
        if let Some(initializer) = initializer {
            statements.push(*initializer);
        }
        statements.push(loop_body);

        Ok(Statement::BlockStmt(statements))
    }
    fn while_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.expect(&TokenType::RightParen)?;
        let body = self.statement()?;
        Ok(Statement::WhileStmt(condition, Box::new(body)))
    }
    fn if_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.expect(&TokenType::RightParen)?;

        let then_branch = Box::from(self.statement()?);
        let mut else_branch: Option<Box<Statement>> = None;
        if self.match_any(&[TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }
        Ok(Statement::IfStmt(condition, then_branch, else_branch))
    }
    fn block_statement(&mut self) -> Result<Statement, ParseError> {
        let mut stmts = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        self.expect(&TokenType::RightBrace)?;
        Ok(Statement::BlockStmt(stmts))
    }
    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        self.expect(&TokenType::Semicolon)?;
        Ok(Statement::ExpressionStmt(expr))
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        self.expect(&TokenType::Semicolon)?;
        Ok(Statement::PrintStmt(expr))
    }

    fn declaration(&mut self) -> Result<Statement, ParseError> {
        if self.match_any(&[TokenType::Var]) {
            self.var_declaration()
        } else if self.match_any(&[TokenType::Fun]) {
            let name = self.expect(&TokenType::Identifier)?.clone();
            self.function(name, FunctionType::FUNCTION)
        } else if self.match_any(&[TokenType::Class]) {
            self.class_declaration()
        } else {
            self.statement()
        }
    }
    fn class_declaration(&mut self) -> Result<Statement, ParseError> {
        let name = self.expect(&TokenType::Identifier)?.clone();
        let mut super_class = None;
        if self.match_any(&[TokenType::Less]) {
            super_class = Some(Expr::Variable(self.expect(&TokenType::Identifier)?.clone()));
        }
        self.match_any(&[TokenType::LeftBrace]);
        let mut methods = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let name = self.expect(&TokenType::Identifier)?.clone();
            let function_type;
            if name.lexeme.to_lowercase() == "init" {
                function_type = FunctionType::INIT;
            } else {
                function_type = FunctionType::METHOD;
            }
            methods.push(self.function(name, function_type)?);
        }
        self.expect(&TokenType::RightBrace)?;
        Ok(Statement::ClassStmt(name, methods, super_class))
    }

    fn function(
        &mut self,
        name: Token,
        function_type: FunctionType,
    ) -> Result<Statement, ParseError> {
        self.expect(&TokenType::LeftParen)?;
        let mut params: Vec<Token> = vec![];
        if !self.check(&TokenType::RightParen) {
            params.push(self.expect(&TokenType::Identifier)?.clone());
            while self.match_any(&[TokenType::Comma]) {
                params.push(self.expect(&TokenType::Identifier)?.clone());
            }
        }
        self.expect(&TokenType::RightParen)?;
        self.expect(&TokenType::LeftBrace)?;
        let body = self.block_statement()?;
        Ok(Statement::FunctionStmt(
            name,
            params,
            Rc::new(Box::new(body)),
            function_type,
        ))
    }
    fn var_declaration(&mut self) -> Result<Statement, ParseError> {
        let name = self.expect(&TokenType::Identifier)?.clone();
        let initializer = if self.match_any(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.expect(&TokenType::Semicolon)?;
        Ok(Statement::VarStmt(name, initializer))
    }
}
