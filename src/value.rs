use std::{fmt::Display, rc::Rc};
use std::fmt::Debug;

use crate::chunk::Chunk;

 #[derive(Clone, PartialEq, Debug)]
 pub enum Value {
    Bool(bool),
    Number(f64),
    Null,
    String(Rc<String>),
    Func(Rc<Function>),
    NativeFunc(Rc<NativeFunction>),
 }
 
 #[derive(Clone, PartialEq, Debug)]
 pub struct Function {
    pub name: String,
    pub arity: u8,
    pub chunk: Chunk
 }

pub struct NativeFunction {
    pub name: String,
    pub arity: u8,
   pub function: Box<dyn Fn(&[Value]) -> Value>,
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFunction").field("name", &self.name).field("arity", &self.arity).finish()
    }
}




 impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Null => write!(f, "NULL"),
            Value::String(v) => write!(f, "{}", v),
            Value::Func(function) => write!(f, "fn {}", function.name),
            Value::NativeFunc(native_function) => write!(f, "fn {}", native_function.name),
        }
    }
 }