use crate::lexer::token::Token;
use crate::parser::expr::Expr;
use crate::parser::stmt::{FunctionType, Statement};
use std::collections::HashMap;
use std::rc::Rc;

pub struct Resolver {
    scopes: Vec<Vec<(String, bool)>>,
    pub locals: HashMap<*const Expr, (usize, usize)>, // distance and index
    current_function: FunctionType,
    pub had_error: bool,
}
impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            locals: HashMap::new(),
            current_function: FunctionType::NONE,
            had_error: false,
        }
    }
    pub fn resolve_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::BlockStmt(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.resolve_statement(stmt);
                }
                self.end_scope();
            }
            Statement::VarStmt(name, expr) => {
                self.declare(name);
                if let Some(expr) = expr {
                    self.resolve_expr(expr);
                }
                self.define(name);
            }
            Statement::FunctionStmt(identifier, params, body, function_type) => {
                self.declare(identifier);
                self.define(identifier);
                self.resolve_function(params, body, function_type.clone());
            }
            Statement::ExpressionStmt(expr) => {
                self.resolve_expr(expr);
            }
            Statement::IfStmt(condition, then_branch, else_branch) => {
                self.resolve_expr(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch);
                }
            }
            Statement::PrintStmt(expr) => {
                self.resolve_expr(expr);
            }
            Statement::ReturnStmt(_, expr) => {
                if self.current_function == FunctionType::NONE {
                    eprintln!("return used without function");
                    self.had_error = true;
                    return;
                }
                if let Some(e) = expr {
                    self.resolve_expr(e)
                }
            }
            Statement::WhileStmt(condition, body) => {
                self.resolve_expr(condition);
                self.resolve_statement(body);
            }
            Statement::ClassStmt(name, methods) => {
                self.declare(name);
                self.define(name);
                self.begin_scope();
                self.scopes
                    .last_mut()
                    .unwrap()
                    .push(("this".to_string(), true)); // add "this" variable
                for method in methods {
                    self.resolve_statement(method);
                }
                self.end_scope();
            }
        }
    }
    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Rc<Box<Statement>>,
        function_type: FunctionType,
    ) {
        let enclosing_function = self.current_function.clone();
        self.current_function = function_type;
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        if let Statement::BlockStmt(stmts) = body.as_ref().as_ref() {
            for stmt in stmts {
                self.resolve_statement(stmt);
            }
        }
        self.end_scope();
        self.current_function = enclosing_function;
    }

    pub fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.iter().any(|(n, _)| n == &name.lexeme) {
            eprintln!(
                "Variable '{}' was redefined in line {}",
                name.lexeme, name.line
            );
            self.had_error = true;
            return;
        }
        scope.push((name.lexeme.clone(), false));
    }
    pub fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        if let Some((_, is_ready)) = scope.iter_mut().rev().find(|(n, _)| n == &name.lexeme) {
            *is_ready = true;
        }
    }
    pub fn begin_scope(&mut self) {
        self.scopes.push(vec![])
    }
    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_expr(&mut self, expr: &Expr) {
        match &expr {
            Expr::Variable(token) => {
                if let Some(scope) = self.scopes.last() {
                    if let Some((_, false)) = scope.iter().rev().find(|(n, _)| n == &token.lexeme) {
                        eprintln!(
                            "Can't read local variable in its own initializer at line {}",
                            token.line
                        );
                        self.had_error = true;
                        return;
                    }
                }
                self.resolve_local(expr, token);
            }
            Expr::Assignment { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
            }
            Expr::Binary {
                left,
                operator: _operator,
                right,
            } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Call {
                callee,
                token: _token,
                args,
            } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::Grouping { expr } => {
                self.resolve_expr(expr);
            }
            Expr::Literal(_) => {}
            Expr::Logical {
                left,
                operator: _operator,
                right,
            } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Unary {
                operator: _operator,
                right,
            } => {
                self.resolve_expr(right);
            }
            Expr::Get {
                object,
                name: _name,
            } => {
                self.resolve_expr(object);
            }
            Expr::Set {
                object,
                name: _name,
                value,
            } => {
                self.resolve_expr(object);
                self.resolve_expr(value);
            }
            Expr::This { token } => {
                self.resolve_local(expr, token);
            }
        }
    }

    fn resolve_local(&mut self, expr: &Expr, token: &Token) {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            for (index, (name, _)) in scope.iter().enumerate().rev() {
                if name.as_str() == token.lexeme.as_str() {
                    // match the variable name
                    // depth is how many environments to jump.
                    // index is the slot in the environment's Vec.
                    self.locals.insert(expr as *const Expr, (depth, index));
                    return;
                }
            }
        }
    }
}
