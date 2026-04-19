use crate::interpreter::function::LoxCallable;
use std::fmt::Formatter;
use std::rc::Rc;


#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
    Callable(Rc<dyn LoxCallable>),
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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
