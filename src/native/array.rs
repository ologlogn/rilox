use crate::error::Error;
use crate::interpreter::function::LoxCallable;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub struct ArrayFn;

impl Debug for ArrayFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn Array")
    }
}

impl LoxCallable for ArrayFn {
    fn arity(&self) -> usize {
        0
    }
    fn call(&self, _interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error> {
        let mut array = vec![];
        for arg in args {
            array.push(arg)
        }
        Ok(Value::Array(Rc::new(RefCell::new(array))))
    }
    fn is_variadic(&self) -> bool {
        true
    }
}

pub struct ArrayMethod {
    pub method_name: String,
    pub array: Rc<RefCell<Vec<Value>>>,
}

impl Debug for ArrayMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}()", self.method_name)
    }
}
fn get_index(value: Value) -> Result<usize, Error> {
    match value {
        Value::Number(num) => {
            if num.fract() != 0.0 || num < 0.0 {
                return Err(Error::RuntimeError(
                    "Array index must be a non-negative integer.".to_string(),
                ));
            }
            Ok(num as usize)
        }
        _ => Err(Error::RuntimeError(
            "Array index must be a number.".to_string(),
        )),
    }
}

impl LoxCallable for ArrayMethod {
    fn arity(&self) -> usize {
        match self.method_name.as_str() {
            "push" => 1,
            "pop" => 0,
            "len" => 0,
            "set" => 2,
            "get" => 1,
            _ => 0,
        }
    }

    fn call(&self, _interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error> {
        let mut arr = self.array.borrow_mut();
        match self.method_name.as_str() {
            "push" => {
                arr.push(args[0].clone());
                Ok(Value::Nil)
            }
            "pop" => arr.pop().ok_or(Error::RuntimeError(
                "Cannot pop from empty array.".to_string(),
            )),
            "len" => Ok(Value::Number(arr.len() as f64)),
            "set" => {
                let index = get_index(args[0].clone())?;
                if index >= arr.len() {
                    Err(Error::RuntimeError("Cannot set array index.".to_string()))
                } else {
                    arr[index] = args[1].clone();
                    Ok(arr[index].clone())
                }
            }
            "get" => {
                let index = get_index(args[0].clone())?;
                if index >= arr.len() {
                    Err(Error::RuntimeError("Cannot set array index.".to_string()))
                } else {
                    Ok(arr[index].clone())
                }
            }
            _ => Err(Error::RuntimeError(format!(
                "Undefined array method '{}'.",
                self.method_name
            ))),
        }
    }
}
