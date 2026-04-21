use crate::interpreter::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone)]
pub struct Environment {
    map: HashMap<String, Value>,
    parent: Option<EnvRef>,
    values: Vec<Value>,
}
impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            map: HashMap::<String, Value>::new(),
            parent,
            values: Vec::new(),
        }
    }
    pub fn define(&mut self, name: String, value: Value) {
        if self.parent.is_none() {
            self.map.insert(name, value);
        } else {
            self.values.push(value); // it will match resolver index
        }
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

    pub fn assign(&mut self, name: String, value: Value) -> bool {
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
    pub fn get_at(env: EnvRef, distance: usize, index: usize) -> Value {
        let ancestor = Self::ancestor(env, distance);
        ancestor.borrow().values[index].clone()
    }


    pub fn assign_at(env: EnvRef, distance: usize, index: usize, value: Value) {
        let ancestor = Self::ancestor(env, distance);
        ancestor.borrow_mut().values[index] = value;
    }

}
