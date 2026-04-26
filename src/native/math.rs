use crate::error::Error;
use crate::interpreter::function::LoxCallable;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;

#[derive(Debug)]
pub struct MathFloorFn;

impl LoxCallable for MathFloorFn {
    fn arity(&self) -> usize { 1 }

    fn call(&self, _interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error> {
        match args[0] {
            Value::Number(num) => Ok(Value::Number(num.floor())), 
            _ => Err(Error::RuntimeError("math.floor requires a number.".to_string())),
        }
    }
}
