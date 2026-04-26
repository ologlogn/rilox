use crate::interpreter::class::LoxClass;
use crate::interpreter::function::LoxCallable;
use crate::interpreter::instance::LoxInstance;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
    Callable(Rc<dyn LoxCallable>),      // dyn because of trait
    Class(Rc<LoxClass>),                // immutable definition
    Instance(Rc<RefCell<LoxInstance>>), // RefCell because mutable state
    Array(Rc<RefCell<Vec<Value>>>),
}
pub fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Nil => false,
        Value::Boolean(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Callable(_) => write!(f, "<fn>"),
            Value::Class(class) => write!(f, "<class> {}", class.name()),
            Value::Instance(instance) => {
                write!(f, "<instance> of class {}", instance.borrow().class.name())
            }
            Value::Array(array) => {
                write!(f, "[")?;
                for (index, value) in array.borrow().iter().enumerate() {
                    if index == array.borrow().len() - 1 {
                        write!(f, "{}", value)?;
                    } else {
                        write!(f, "{},", value)?
                    }
                }
                write!(f, "]")
            }
        }
    }
}

pub fn is_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(na), Value::Number(nb)) => na == nb,
        (Value::Boolean(ba), Value::Boolean(bb)) => ba == bb,
        (Value::Nil, Value::Nil) => true,
        (Value::String(sa), Value::String(sb)) => sa == sb,
        _ => false,
    }
}
