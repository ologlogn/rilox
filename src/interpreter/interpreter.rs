use crate::interpreter::value::{Value, is_equal, is_truthy};
use crate::error::{Error, runtime_error};
use crate::interpreter::env::{EnvRef, Environment};
use crate::interpreter::function::{ClockFn, LoxFunction};
use crate::lexer::token::{Literal, TokenType};
use crate::parser::expr::Expr;
use crate::parser::stmt::Statement;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
    pub(crate) globals: EnvRef,
    pub(crate) env: EnvRef,
    pub(crate) had_error: bool,
    pub(crate) locals: HashMap<*const Expr, (usize,usize) >,
}

impl Interpreter {
    pub fn execute_stmt(&mut self, stmt: &Statement) -> Result<(), Error> {
        match stmt {
            Statement::ExpressionStmt(expr) => {
                self.eval_expr(expr);
                Ok(())
            }

            Statement::PrintStmt(expr) => {
                let val = self.eval_expr(expr);
                println!("{}", val);
                Ok(())
            }

            Statement::VarStmt(identifier, expr) => {
                let val = if let Some(ex) = expr {
                    self.eval_expr(ex)
                } else {
                    Value::Nil
                };
                self.env.borrow_mut().define(identifier.lexeme.clone(), val);
                Ok(())
            }

            Statement::BlockStmt(stmts) => {
                let new_env = Rc::new(RefCell::new(Environment::new(Some(self.env.clone()))));
                self.execute_block(stmts, new_env)
            }

            Statement::IfStmt(cond, if_branch, else_branch) => {
                if is_truthy(&self.eval_expr(cond)) {
                    self.execute_stmt(if_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute_stmt(else_branch)
                } else {
                    Ok(())
                }
            }

            Statement::WhileStmt(cond, body) => {
                while is_truthy(&self.eval_expr(cond)) {
                    self.execute_stmt(body)?;
                }
                Ok(())
            }

            Statement::FunctionStmt(identifier, params, body) => {
                let func = LoxFunction {
                    params: params.clone(),
                    body: body.clone(), // Rc clone — same pointers
                    closure: self.env.clone(),
                };
                self.env.borrow_mut().define(identifier.lexeme.clone(), Value::Callable(Rc::new(func)));
                Ok(())
            }

            Statement::ReturnStmt(_, expr) => {
                let val = if let Some(e) = expr {
                    self.eval_expr(e)
                } else {
                    Value::Nil
                };
                Err(Error::Return(val))
            }
        }
    }

    pub(crate) fn execute_block(
        &mut self,
        stmts: &[Statement],
        new_env: EnvRef,
    ) -> Result<(), Error> {
        let previous = self.env.clone();
        self.env = new_env;
        let result = stmts.iter().try_for_each(|s| self.execute_stmt(s));
        self.env = previous;
        result
    }
    pub(crate) fn execute_stmt_block(&mut self, stmt: &Statement, new_env: EnvRef) -> Result<(), Error> {
        if let Statement::BlockStmt(stmts) = stmt {
            self.execute_block(stmts, new_env)
        } else {
            panic!("expected block");
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => Value::Number(*n),
                Literal::Boolean(b) => Value::Boolean(*b),
                Literal::String(s) => Value::String(s.clone()),
                Literal::Nil => Value::Nil,
            },
            Expr::Unary { operator, right } => {
                let val = self.eval_expr(right);
                match operator.token_type {
                    TokenType::Minus => {
                        if let Value::Number(n) = val {
                            Value::Number(-n)
                        } else {
                            runtime_error(operator.clone(), "Operands must be numbers");
                            Value::Nil
                        }
                    }
                    TokenType::Bang => Value::Boolean(!is_truthy(&val)),
                    _ => Value::Nil,
                }
            }
            Expr::Grouping { expr } => self.eval_expr(expr),
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left = &self.eval_expr(left);
                let right = &self.eval_expr(right);
                match operator.token_type {
                    TokenType::BangEqual => Value::Boolean(!is_equal(left, right)),
                    TokenType::EqualEqual => Value::Boolean(is_equal(left, right)),
                    TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual => {
                        if let (Value::Number(l), Value::Number(r)) = (left, right) {
                            match operator.token_type {
                                TokenType::Greater => Value::Boolean(l > r),
                                TokenType::GreaterEqual => Value::Boolean(l >= r),
                                TokenType::Less => Value::Boolean(l < r),
                                TokenType::LessEqual => Value::Boolean(l <= r),
                                _ => unreachable!(),
                            }
                        } else {
                            runtime_error(operator.clone(), "Operands must be numbers.");
                            Value::Nil
                        }
                    }
                    TokenType::Plus => {
                        if let (Value::Number(l), Value::Number(r)) = (left, right) {
                            Value::Number(l + r)
                        } else if let (Value::String(l), Value::String(r)) = (left, right) {
                            Value::String(format!("{}{}", l, r))
                        } else {
                            runtime_error(
                                operator.clone(),
                                "Operands must be two numbers or two strings.",
                            );
                            Value::Nil
                        }
                    }
                    TokenType::Minus => {
                        if let (Value::Number(l), Value::Number(r)) = (left, right) {
                            Value::Number(l - r)
                        } else {
                            runtime_error(operator.clone(), "Operands must be numbers.");
                            Value::Nil
                        }
                    }
                    TokenType::Star => {
                        if let (Value::Number(l), Value::Number(r)) = (left, right) {
                            Value::Number(l * r)
                        } else {
                            runtime_error(operator.clone(), "Operands must be numbers.");
                            Value::Nil
                        }
                    }
                    TokenType::Slash => {
                        if let (Value::Number(l), Value::Number(r)) = (left, right) {
                            if *r == 0.0 {
                                runtime_error(operator.clone(), "Cannot divide by zero.");
                                return Value::Nil;
                            }
                            Value::Number(l / r)
                        } else {
                            runtime_error(operator.clone(), "Operands must be numbers.");
                            Value::Nil
                        }
                    }
                    _ => Value::Nil,
                }
            }
            Expr::Variable(var_token) => self.lookup(&var_token.lexeme, expr),
            Expr::Assignment { name, value } => {
                let val = self.eval_expr(value);
                let key = expr as *const Expr;

                // Unpack both distance and index!
                if let Some(&(distance, index)) = self.locals.get(&key) {
                    Environment::assign_at(self.env.clone(), distance, index, val.clone());
                } else {
                    self.globals.borrow_mut().assign(name.lexeme.clone(), val.clone());
                }
                val
            }

            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = &self.eval_expr(left);
                if operator.token_type == TokenType::Or {
                    if is_truthy(left) {
                        return left.clone();
                    }
                } else {
                    if !is_truthy(left) {
                        return left.clone();
                    }
                }
                self.eval_expr(right)
            }
            Expr::Call {
                callee,
                token,
                args,
            } => {
                let callee_val = self.eval_expr(callee);
                let args: Vec<Value> = args.iter().map(|arg| self.eval_expr(arg)).collect();

                if let Value::Callable(func) = callee_val {
                    let func = func.clone();
                    if args.len() != func.arity() {
                        runtime_error(token.clone(), "Wrong number of arguments");
                        return Value::Nil;
                    }
                    func.call(self, args).unwrap_or(Value::Nil)
                } else {
                    runtime_error(token.clone(), "Not a function");
                    Value::Nil
                }
            }
        }
    }


    pub(crate) fn lookup(&self, name: &str, expr: &Expr) -> Value {
        let key = expr as *const Expr;
        // Unpack both distance and index!
        if let Some(&(distance, index)) = self.locals.get(&key) {
            Environment::get_at(self.env.clone(), distance, index)
        } else {
            self.globals.borrow().get(name) // always fall back to globals
        }
    }

    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals
            .borrow_mut()
            .define("clock".to_string(), Value::Callable(Rc::new(ClockFn)));
        Self {
            env: globals.clone(),
            globals,
            had_error: false,
            locals: HashMap::new(),
        }
    }
}
