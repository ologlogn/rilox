use crate::error::{Error, runtime_error};
use crate::interpreter::class::LoxClass;
use crate::interpreter::env::{EnvRef, Environment};
use crate::interpreter::function::{LoxCallable, LoxFunction};
use crate::interpreter::instance::LoxInstance;
use crate::interpreter::value::{Value, is_equal, is_truthy};
use crate::lexer::token::{Literal, TokenType};
use crate::native::array::{ArrayFn, ArrayMethod};
use crate::native::clock::ClockFn;
use crate::native::convert::ToNumberFn;
use crate::native::io::ReadLineFn;
use crate::native::math::MathFloorFn;
use crate::parser::expr::Expr;
use crate::parser::stmt::Statement;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
    pub(crate) globals: EnvRef,
    pub(crate) env: EnvRef,
    pub(crate) had_error: bool,
    pub(crate) locals: HashMap<*const Expr, (usize, usize)>,
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

            Statement::FunctionStmt(identifier, params, body, function_type) => {
                let func = LoxFunction {
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.env.clone(),
                    function_type: function_type.clone(),
                };
                self.env
                    .borrow_mut()
                    .define(identifier.lexeme.clone(), Value::Callable(Rc::new(func)));
                Ok(())
            }

            Statement::ClassStmt(name, methods, super_class) => {
                let mut sup = None;
                if let Some(super_class) = super_class {
                    let class_def = self.eval_expr(super_class);
                    if let Value::Class(class_def) = class_def {
                        sup = Some(class_def);
                    } else {
                        runtime_error(name.clone(), "Not a class")
                    }
                }
                let mut closure_env = self.env.clone();
                if let Some(ref superclass_def) = sup {
                    let mut super_env = Environment::new(Some(self.env.clone()));
                    super_env.define("super".to_string(), Value::Class(superclass_def.clone()));
                    closure_env = Rc::new(RefCell::new(super_env));
                }
                let mut class_methods = HashMap::new();
                for method in methods {
                    if let Statement::FunctionStmt(method_name, params, body, function_type) =
                        method
                    {
                        let function = LoxFunction {
                            params: params.clone(),
                            body: body.clone(),
                            closure: closure_env.clone(),
                            function_type: function_type.clone(),
                        };
                        class_methods.insert(method_name.lexeme.clone(), Rc::new(function));
                    }
                }

                let class = LoxClass::new(name.lexeme.clone(), class_methods, sup);
                self.env
                    .borrow_mut()
                    .define(name.lexeme.clone(), Value::Class(Rc::new(class)));
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
    pub(crate) fn execute_stmt_block(
        &mut self,
        stmt: &Statement,
        new_env: EnvRef,
    ) -> Result<(), Error> {
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
                        } else if let (Value::Number(l), Value::String(r)) = (left, right) {
                            Value::String(format!("{}{}", l, r))
                        } else if let (Value::String(l), Value::Number(r)) = (left, right) {
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
                    self.globals
                        .borrow_mut()
                        .assign(name.lexeme.clone(), val.clone());
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

                match callee_val {
                    Value::Callable(func) => {
                        let func = func.clone();
                        if !func.is_variadic() && args.len() != func.arity() {
                            runtime_error(token.clone(), "Wrong number of arguments");
                            return Value::Nil;
                        }
                        func.call(self, args).unwrap_or(Value::Nil)
                    }
                    Value::Class(class) => {
                        if args.len() != class.arity() {
                            runtime_error(token.clone(), "Wrong number of arguments");
                            return Value::Nil;
                        }
                        class.call(self, args).unwrap_or(Value::Nil)
                    }
                    _ => {
                        runtime_error(token.clone(), "Not a function");
                        Value::Nil
                    }
                }
            }
            Expr::Get { object, name } => {
                let ob = self.eval_expr(object);
                if let Value::Instance(instance) = ob {
                    LoxInstance::get(&instance, name)
                } else if let Value::Array(arr) = ob {
                    if ["push", "pop", "len", "get", "set"].contains(&&*name.lexeme) {
                        return Value::Callable(Rc::new(ArrayMethod {
                            method_name: name.lexeme.to_string(),
                            array: arr,
                        }));
                    } else {
                        runtime_error(name.clone(), "not a function");
                        Value::Nil
                    }
                } else {
                    runtime_error(name.clone(), "Not a instance");
                    Value::Nil
                }
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                let ob = self.eval_expr(object);
                if let Value::Instance(instance) = ob {
                    let val = self.eval_expr(value);
                    instance.borrow_mut().set(name.lexeme.clone(), val.clone());
                    val
                } else {
                    runtime_error(name.clone(), "Not a instance");
                    Value::Nil
                }
            }
            Expr::This { token } => self.lookup(&token.lexeme, expr),
            Expr::Super { keyword, method } => {
                // EVALUATING: `super.method`
                // We need TWO pieces of context to evaluate a super call:
                // 1. The superclass itself (to find the actual method definition).
                // 2. The current instance "this" (to bind to the method so it executes on this object).
                //
                // Environment Layout:
                // When class methods are created, we wrap them in an environment containing "super".
                // When a method is called, we wrap it again in an environment containing "this".
                // Therefore, if the resolver says "super" is at `distance`, we know "this" is
                // exactly one level closer at `distance - 1`. Because both are pushed to local
                // scopes, they sit safely at index 0 of their respective environment's values vector.
                let key = expr as *const Expr;
                let distance = if let Some(&(dist, _idx)) = self.locals.get(&key) {
                    dist
                } else {
                    runtime_error(keyword.clone(), "Super expression not resolved.");
                    return Value::Nil;
                };
                let superclass_val = Environment::get_at(self.env.clone(), distance, 0);
                let superclass = if let Value::Class(c) = superclass_val {
                    c
                } else {
                    runtime_error(keyword.clone(), "Superclass must be a class.");
                    return Value::Nil;
                };
                let object_val = Environment::get_at(self.env.clone(), distance - 1, 0);
                let instance = if let Value::Instance(inst) = object_val {
                    inst
                } else {
                    runtime_error(keyword.clone(), "Failed to extract 'this' instance.");
                    return Value::Nil;
                };
                if let Some(method_func) = superclass.find_method(method.lexeme.clone()) {
                    method_func.bind(instance)
                } else {
                    runtime_error(
                        method.clone(),
                        &format!("Undefined property '{}'.", method.lexeme),
                    );
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
        let mut instance;
        let env = Rc::new(RefCell::new(Environment::new(None)));
        instance = Self {
            env: env.clone(),
            globals: env,
            had_error: false,
            locals: HashMap::new(),
        };
        instance.define_global();
        instance
    }
    fn define_global(&mut self) {
        let mut globals = self.globals.borrow_mut();
        globals.define("clock".to_string(), Value::Callable(Rc::new(ClockFn)));
        globals.define(
            "read_line".to_string(),
            Value::Callable(Rc::new(ReadLineFn)),
        );
        globals.define(
            "to_number".to_string(),
            Value::Callable(Rc::new(ToNumberFn)),
        );
        globals.define("array".to_string(), Value::Callable(Rc::new(ArrayFn)));
        globals.define("floor".to_string(), Value::Callable(Rc::new(MathFloorFn)))
    }
}
