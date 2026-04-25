use crate::error::runtime_error;
use crate::interpreter::class::LoxClass;
use crate::interpreter::function::LoxCallable;
use crate::interpreter::value::Value;
use crate::lexer::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LoxInstance {
    pub class: Rc<RefCell<LoxClass>>,
    pub fields: HashMap<String, Value>,
}
impl LoxInstance {
    pub fn get(&self, name: &Token) -> Value {
        let key = name.lexeme.as_str();
        if self.fields.contains_key(key) {
            return self.fields[key].clone();
        }
        if let Some(method) = self.class.borrow().find_method(name.lexeme.clone()) {
            return Value::Callable(Rc::new(method.borrow().clone()));
        }
        runtime_error(name.clone(), "Undefined property");
        Value::Nil
    }
    pub fn set(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
    }
}
