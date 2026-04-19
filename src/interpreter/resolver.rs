use crate::lexer::token::Token;
use crate::parser::expr::Expr;
use crate::parser::stmt::Statement;
use std::collections::HashMap;

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    pub locals: HashMap<*const Expr, usize>,
}
impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            locals: HashMap::new(),
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
                self.declare(name.clone());
                if let Some(expr) = expr {
                    self.resolve_expr(expr);
                }
                self.define(name.clone());
            }
            Statement::FunctionStmt(identifier, params, body) => {
                self.declare(identifier.lexeme.clone());
                self.define(identifier.lexeme.clone());
                self.resolve_function(params, body);
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
                if let Some(e) = expr {
                    self.resolve_expr(e)
                }
            }
            Statement::WhileStmt(condition, body) => {
                self.resolve_expr(condition);
                self.resolve_statement(body);
            }
        }
    }
    fn resolve_function(&mut self, params: &Vec<Token>, body: &Box<Statement>) {
        self.begin_scope();
        for param in params {
            self.declare(param.lexeme.clone());
            self.define(param.lexeme.clone())
        }
        self.resolve_statement(body);
        self.end_scope();
    }
    fn declare(&mut self, name: String) {
        if !self.scopes.is_empty() {
            let scope: &mut HashMap<String, bool> = self.scopes.last_mut().unwrap();
            scope.insert(name, false);
        }
    }
    fn define(&mut self, name: String) {
        if !self.scopes.is_empty() {
            let scope: &mut HashMap<String, bool> = self.scopes.last_mut().unwrap();
            scope.insert(name, true);
        }
    }
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_expr(&mut self, expr: &Expr) {
        match &expr {
            Expr::Variable(token) => {
                if let Some(scope) = self.scopes.last() {
                    if let Some(false) = scope.get(token.lexeme.as_str()).copied() {
                        panic!("Can't read local variable in its own initializer");
                    }
                }
                self.resolve_local(expr, token);
            }
            Expr::Assignment { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
            }
            Expr::Binary {left, operator: _operator, right} => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Call {callee, token: _token, args} =>  {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::Grouping {expr} => {
                self.resolve_expr(expr);
            }
            Expr::Literal(_) => {}
            Expr::Logical {left, operator: _operator, right} => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Unary { operator: _operator, right} => {
                self.resolve_expr(right);
            }
        }
    }

    fn resolve_local(&mut self, expr: &Expr, token: &Token) {
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(token.lexeme.as_str()) {
                self.locals.insert(expr as *const Expr, self.scopes.len() - 1 - index);
                return;
            }
        }
    }
}
