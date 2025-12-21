use std::{fmt::Display, rc::Rc};

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

 #[derive(Clone, Debug, PartialEq)]
 pub struct NativeFunction {
    pub name: String,
    pub arity: u8,
    pub function: fn(&[Value]) -> Value
 }


 impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Null => write!(f, "NULL"),
            Value::String(v) => write!(f, "{}", v),
            Value::Func(function) => write!(f, "{}", function.name),
            Value::NativeFunc(native_function) => write!(f, "{}", native_function.name),
        }
    }
 }