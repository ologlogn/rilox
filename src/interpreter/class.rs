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
        let initializer = self.find_method("init".to_string());
        if let Some(init) = initializer {
            init.borrow().arity()
        } else {
            0
        }
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error> {
        let instance = Rc::new(RefCell::new(LoxInstance {
            class: Rc::new(self.clone()),
            fields: HashMap::new(),
        }));

        if let Some(initializer) = self.find_method("init".to_string()) {
            let bound_value = initializer.borrow().bind(instance.clone());
            if let Value::Callable(bound_init) = bound_value {
                bound_init.call(interpreter, args)?;
            }
        }
        Ok(Value::Instance(instance))
    }
}
