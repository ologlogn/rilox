use crate::error::{runtime_error};
use crate::interpreter::class::LoxClass;
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
            self.fields[key].clone()
        } else {
            runtime_error(name.clone(), "Undefined property");
            Value::Nil
        }
    }
}
