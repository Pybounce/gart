use std::time::{SystemTime, UNIX_EPOCH};

use crate::{compiler::Compiler, value::{NativeFunction, Value}, vm::VM};

pub struct Interpreter {
    vm: VM
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
    pub fn new(source: String, natives: Vec<NativeFunction>) -> Result<Self, Vec<CompilerError>> {
        let mut compiler = Compiler::new(&source);

        add_builtin_natives(&mut compiler);

        for native in natives.into_iter() {
            compiler.add_native(native);
        }

        match compiler.compile() {
            Ok(compiler_out) => {
                let vm = VM::new(compiler_out);

                let interpreter = Self {
                    vm: vm,
                };
                return Ok(interpreter);

            },
            Err(compiler_errors) => {
                return Err(compiler_errors);
            },
        }

    }
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        return match self.vm.run() {
            Ok(()) => Ok(()),
            Err(runtime_err) => Err(runtime_err),
        };
    }

    /// Returns boolean for if there's a next step </br>
    /// False means there will be no next step.
    pub fn step(&mut self) -> Result<bool, RuntimeError> {
        return self.vm.step();
    }

}

    fn add_builtin_natives(compiler: &mut Compiler) {
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
                Box::new(time)
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
                Box::new(print)
            },
        };

        compiler.add_native(time);
        compiler.add_native(print);
    }