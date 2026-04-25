use std::cell::RefCell;
use std::rc::Rc;
use crate::interpreter::class::LoxClass;

#[derive(Clone, Debug)]
pub struct LoxInstance {
    pub class: Rc<RefCell<LoxClass>>,
}