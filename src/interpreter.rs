use std::{io::{self, Write}, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use rand::Rng;

use crate::{compiler::Compiler, value::{NativeFunction, Value}, vm::VM};

pub struct Interpreter {
    vm: VM
}

pub struct RuntimeError {
    pub message: &'static str
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerError {
    pub line: usize,
    pub start: usize,
    pub len: usize,
    pub message: String
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

        let random_range = NativeFunction {
            name: "random_range".to_owned(),
            arity: 2,
            function: {
                fn random_range(vals: &[Value]) -> Value {
                    return match (vals[0].clone(), vals[1].clone()) {
                        (Value::Number(min), Value::Number(max)) => {
                            let mut rng = rand::rng();
                            return Value::Number(rng.random_range(min..=max) as f64);
                        },
                        _ => Value::Null
                    };

                }

                Box::new(random_range)
            },
        };

        let to_number = NativeFunction {
            name: "number".to_owned(),
            arity: 1,
            function: {
                fn number(vals: &[Value]) -> Value {
                    return match vals[0].clone() {
                        Value::Number(number) => Value::Number(number),
                        Value::String(string) => match string.parse::<f64>() {
                            Ok(number) => Value::Number(number),
                            Err(_) => Value::Null,
                        },
                        _ => Value::Null
                    };
                }
                Box::new(number)
            },
        };

        let to_string = NativeFunction {
            name: "string".to_owned(),
            arity: 1,
            function: {
                fn to_string(vals: &[Value]) -> Value {
                    match vals[0].clone() {
                        Value::String(s) => Value::String(s),
                        Value::Number(n) => Value::String(Rc::new(n.to_string())),
                        Value::Bool(b) => Value::String(Rc::new(b.to_string())),
                        Value::Null => Value::String(Rc::new("null".to_owned())),
                        _ => Value::Null,
                    }
                }
                Box::new(to_string)
            },
        };

        let get_input = NativeFunction {
            name: "input".to_owned(),
            arity: 1,
            function: {
                fn input(vals: &[Value]) -> Value {
                    print!("{}", vals[0].clone());
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    return Value::String(Rc::new(input.trim().to_string()));
                }
                Box::new(input)
            },
        };

        let clear = NativeFunction {
            name: "clear".to_owned(),
            arity: 0,
            function: {
                fn clear(_: &[Value]) -> Value {
                    print!("\x1B[2J\x1B[1;1H"); // basically regex so who the fuck knows
                    return Value::Null;
                }
                Box::new(clear)
            },
        };

        let round = NativeFunction {
            name: "round".to_owned(),
            arity: 1,
            function: {
                fn round(vals: &[Value]) -> Value {
                    return match vals[0].clone() {
                        Value::Number(num) => Value::Number(num.round()),
                        _ => Value::Null
                    };
                }
                Box::new(round)
            },
        };

        compiler.add_native(time);
        compiler.add_native(print);
        compiler.add_native(random_range);
        compiler.add_native(to_number);
        compiler.add_native(to_string);
        compiler.add_native(get_input);
        compiler.add_native(clear);
        compiler.add_native(round);
    }