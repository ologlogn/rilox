use crate::error::Error;
use crate::interpreter::function::{LoxCallable, LoxFunction};
use crate::interpreter::instance::LoxInstance;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LoxClass {
    pub(crate) name: String,
    pub(crate) methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
}
impl LoxClass {
    pub fn new(name: String, class_methods: HashMap<String, Rc<RefCell<LoxFunction>>>) -> LoxClass {
        LoxClass {
            name,
            methods: class_methods,
        }
    }
    pub fn find_method(&self, name: String) -> Option<Rc<RefCell<LoxFunction>>> {
        self.methods.get(&name).cloned()
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
            class: Rc::new(self.clone()),
            fields: HashMap::new(),
        };
        Ok(Value::Instance(Rc::new(RefCell::new(instance))))
    }
}
