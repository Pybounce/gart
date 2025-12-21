use std::time::{SystemTime, UNIX_EPOCH};

use crate::{compiler::Compiler, value::{NativeFunction, Value}, vm::VM};

pub struct Interpreter {
    source: String,
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
            source,
        }
    }
    pub fn interpret(&mut self, natives: Vec<NativeFunction>) -> InterpretResult {

        let mut compiler = Compiler::new(&self.source);

        self.add_builtin_natives(&mut compiler);

        for native in natives.into_iter() {
            self.add_native(native, &mut compiler);
        }

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

    pub fn add_native(&self, native: NativeFunction, compiler: &mut Compiler) {
        compiler.add_native(native);
    }

    fn add_builtin_natives(&self, compiler: &mut Compiler) {
        let time = NativeFunction {
            name: "time".to_owned(),
            arity: 0,
            function: {
                fn time(_: &[Value]) -> Value {
                    let secs = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    Value::Number(secs)
                }
                time
            },
        };

        let print = NativeFunction {
            name: "print".to_owned(),
            arity: 1,
            function: {
                fn print(vals: &[Value]) -> Value {
                    println!("{}", vals[0]);
                    return Value::Null;
                }
                print
            },
        };

        self.add_native(time, compiler);
        self.add_native(print, compiler);
    }
}