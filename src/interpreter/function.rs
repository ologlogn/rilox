use crate::error::Error;
use crate::interpreter::env::{EnvRef, Environment};
use crate::interpreter::instance::LoxInstance;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::lexer::token::Token;
use crate::parser::stmt::{FunctionType, Statement};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub trait LoxCallable: Debug {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error>;
}

pub struct ClockFn;

impl Debug for ClockFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn clock")
    }
}

impl LoxCallable for ClockFn {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value, Error> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Ok(Value::Number(time))
    }
}

#[derive(Clone)]
pub struct LoxFunction {
    pub(crate) params: Vec<Token>,
    pub(crate) body: Rc<Box<Statement>>, // shared, never cloned
    pub(crate) closure: EnvRef,
    pub(crate) function_type: FunctionType,
}

impl Debug for LoxFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn>")
    }
}

impl LoxFunction {
    pub fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> Value {
        let mut env = Environment::new(Some(self.closure.clone()));
        env.define("this".to_string(), Value::Instance(instance));
        let bound_function = LoxFunction {
            params: self.params.clone(),
            body: Rc::clone(&self.body),
            closure: Rc::new(RefCell::new(env)),
            function_type: self.function_type.clone(),
        };
        Value::Callable(Rc::new(bound_function))
    }
}
impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error> {
        let call_env = Rc::new(RefCell::new(Environment::new(Some(self.closure.clone()))));
        for (param, arg) in self.params.iter().zip(args) {
            call_env.borrow_mut().define(param.lexeme.clone(), arg);
        }
        match interpreter.execute_stmt_block(&self.body, call_env) {
            Ok(_) => {
                if self.function_type == FunctionType::INIT {
                    return Ok(Environment::get_at(self.closure.clone(), 0, 0));
                }
                Ok(Value::Nil)
            }
            Err(Error::Return(val)) => {
                if self.function_type == FunctionType::INIT {
                    return Ok(Environment::get_at(self.closure.clone(), 0, 0));
                }
                Ok(val)
            }
            Err(e) => Err(e),
        }
    }
}
