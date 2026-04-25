use crate::error::runtime_error;
use crate::interpreter::class::LoxClass;
use crate::interpreter::value::Value;
use crate::lexer::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LoxInstance {
    pub class: Rc<LoxClass>,
    pub fields: HashMap<String, Value>,
}
impl LoxInstance {
    pub fn get(instance_rc: &Rc<RefCell<Self>>, name: &Token) -> Value {
        let key = name.lexeme.as_str();
        if instance_rc.borrow().fields.contains_key(key) {
            return instance_rc.borrow().fields[key].clone();
        }
        if let Some(method) = instance_rc.borrow().class.find_method(name.lexeme.clone()) {
            return method.borrow().bind(instance_rc.clone());
        }
        runtime_error(name.clone(), "Undefined property");
        Value::Nil
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
    }
}
