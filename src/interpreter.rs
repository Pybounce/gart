use std::time::{SystemTime, UNIX_EPOCH};

use crate::{compiler::Compiler, value::{NativeFunction, Value}, vm::VM};

pub struct Interpreter {
    source: String

}

pub enum InterpretResult {
    Ok,
    CompileErr(Vec<CompilerError>),
    RuntimeErr(RuntimeError)
}

pub struct RuntimeError {
    pub message: &'static str
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompilerError {
    pub line: usize,
    pub start: usize,
    pub len: usize
}

impl Interpreter {
    pub fn new(source: String) -> Self {
        Self {
            source
        }
    }
    pub fn interpret(&mut self) -> InterpretResult {
        let time_native = NativeFunction {
            name: "time".to_owned(),
            arity: 0,
            function: {
                fn time_native(_: &[Value]) -> Value {
                    let secs = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    Value::Number(secs)
                }
                time_native
            },
        };
        let mut compiler = Compiler::new(&self.source);
        compiler.add_native(time_native);

        match compiler.compile() {
            Ok(compiler_out) => {
                let mut vm = VM::new();
                return match vm.interpret(compiler_out) {
                    Ok(()) => InterpretResult::Ok,
                    Err(runtime_err) => InterpretResult::RuntimeErr(runtime_err),
                };
            },
            Err(compiler_errors) => {
                return InterpretResult::CompileErr(compiler_errors);
            },
        }
    }
}