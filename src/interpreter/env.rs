use crate::common::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone)]
pub struct Environment {
    map: HashMap<String, Value>,
    parent: Option<EnvRef>,
}
impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            map: HashMap::<String, Value>::new(),
            parent,
        }
    }
    pub fn define(&mut self, name: String, value: Value) {
        self.map.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Value {
        if let Some(value) = self.map.get(name) {
            return value.clone();
        }
        if let Some(parent) = self.parent.as_ref() {
            return parent.borrow().get(name);
        }
        Value::Nil
    }

    pub fn assign(&mut self, name: String, value: &Value) -> bool {
        if self.map.contains_key(&name) {
            self.map.insert(name, value.clone());
            return true;
        }
        if let Some(parent) = self.parent.as_ref() {
            return parent.borrow_mut().assign(name, value);
        }
        false
    }
    pub fn ancestor(env: EnvRef, distance: usize) -> EnvRef {
        let mut current = env;

        for _ in 0..distance {
            let parent = current
                .borrow()
                .parent
                .clone()
                .expect("no ancestor at distance");
            current = parent;
        }
        current
    }
    pub fn get_at(env: EnvRef, distance: usize, name: &str) -> Value {
        let ancestor = Environment::ancestor(env, distance);
        ancestor
            .borrow()
            .map
            .get(name)
            .cloned()
            .unwrap_or(Value::Nil)
    }

    pub fn assign_at(env: EnvRef, distance: usize, name: String, value: &Value) {
        let ancestor = Environment::ancestor(env, distance);
        ancestor.borrow_mut().map.insert(name, value.clone());
    }
}
