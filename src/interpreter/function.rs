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
    fn is_variadic(&self) -> bool {
        false
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
            body: self.body.clone(),
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
        // Init is a special function. it always returns the instance type attached to "this"
        match interpreter.execute_stmt_block(&self.body, call_env) {
            Ok(_) => {
                if self.function_type == FunctionType::INIT {
                    // "this" is always the first one to be stored.
                    // look at bind.
                    return Ok(Environment::get_at(self.closure.clone(), 0, 0));
                }
                Ok(Value::Nil)
            }
            Err(Error::Return(val)) => {
                // Kind of overriding that val. this shouldn't happen because
                // resolver throws errors before running if init function has a return value
                if self.function_type == FunctionType::INIT {
                    return Ok(Environment::get_at(self.closure.clone(), 0, 0));
                }
                Ok(val)
            }
            Err(e) => Err(e),
        }
    }
}
