use crate::{compiler::{Compiler}, vm::{VM}};

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
        let compiler = Compiler::new(&self.source);
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