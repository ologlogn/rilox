use crate::error::Error;
use crate::interpreter::function::LoxCallable;
use crate::interpreter::instance::LoxInstance;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LoxClass {
    pub(crate) name: String,
}
impl LoxClass {
    pub fn new(name: String) -> LoxClass {
        LoxClass { name }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error> {
        let instance = LoxInstance {
            class: Rc::new(RefCell::new(self.clone())),
        };
        Ok(Value::Instance(Rc::new(RefCell::new(instance))))
    }
}
