use crate::{chunk::Chunk, compiler::Compiler, opcode::OpCode, value::Value};


pub struct VM {
    pub pc: usize,
    pub stack: Vec<Value>,
    pub chunk: Chunk,
    pub globals: Vec<Option<Value>>
}

impl VM {
    pub fn new() -> Self {
        Self {
            pc: 0,
            stack: Vec::new(),
            chunk: Chunk::new_terminated(),
            globals: Vec::new()
        }
    }

    pub fn interpret(&mut self, source: &str) -> bool {
        let compiler = Compiler::new(source);
        if let Some(compiler_output) = compiler.compile() {
            self.chunk = compiler_output.chunk;
            self.pc = 0;
            self.globals = vec![None; compiler_output.globals_count];
            return self.run();
        }

        return false;
    }

    fn run(&mut self) -> bool {
        loop {
            let operation = OpCode::try_from(self.read_byte());
            if operation.is_err() { 
                self.runtime_error("Failed to convert byte to opcode");
                return false;
            }
            match operation.unwrap() {
                OpCode::Constant => {
                    let val = self.read_constant();
                    self.stack.push(val);
                },
                OpCode::Print => {
                    println!("{}", self.stack.pop().unwrap())
                },
                OpCode::Pop => { self.stack.pop(); },
                OpCode::Equal => todo!(),
                OpCode::Not => todo!(),
                OpCode::Greater => { if !self.binary_number_op(|a, b| Value::Bool(a > b)) { return false; } },
                OpCode::Less => { if !self.binary_number_op(|a, b| Value::Bool(a < b)) { return false; } },
                OpCode::Add => { if !self.binary_number_op(|a, b| Value::Number(a + b)) { return false; } },
                OpCode::Subtract => { if !self.binary_number_op(|a, b| Value::Number(a - b)) { return false; } },
                OpCode::Multiply => { if !self.binary_number_op(|a, b| Value::Number(a * b)) { return false; } },
                OpCode::Divide => { if !self.binary_number_op(|a, b| Value::Number(a / b)) { return false; } },
                OpCode::Negate => {
                    let val = self.stack.pop().unwrap();
                    if let Value::Number(n) = val {
                        self.stack.push(Value::Number(-n));
                    } else { 
                        self.runtime_error("Negate operand must be number."); 
                        return false;
                    }
                },
                OpCode::Return => return true,
                OpCode::Null => self.stack.push(Value::Null),
                OpCode::DefineGlobal => {
                    self.write_global();
                },
                OpCode::SetLocal => todo!(),
                OpCode::GetLocal => todo!(),
                OpCode::SetGlobal => {
                    self.write_global();
                },
                OpCode::GetGlobal => {
                    match self.read_global() {
                        Some(global_val) => { self.stack.push(global_val); },
                        None => {
                            self.runtime_error("Undefined variable.");
                            return false;
                        },
                    }
    
                },
                OpCode::JumpIfFalse => {
                    let jump = self.read_short() as usize;
                    if self.is_falsey(*self.stack.last().unwrap()) {
                        self.pc += jump;
                    }
                },
                OpCode::Jump => {
                    let jump = self.read_short() as usize;
                    self.pc += jump;
                },
            }
        }
    }
}


// Helpers
impl VM {
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.bytes[self.pc];
        self.pc += 1;
        return byte;
    }
    fn read_short(&mut self) -> u16 {
        self.pc += 2;
        let high = self.chunk.bytes[self.pc - 2] as u16;
        let low = self.chunk.bytes[self.pc - 1] as u16;
        return (high << 8) | low;

    }
    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        return self.chunk.constants[index];
    }
    fn read_global(&mut self) -> Option<Value> {
        let index = self.read_byte() as usize;
        return self.globals[index];
    }
    fn write_global(&mut self) {
        let val = *self.stack.last().unwrap();
        let index = self.read_byte() as usize;
        self.globals[index] = Some(val);
    }
    fn runtime_error(&mut self, message: &'static str) {
        println!("Runtime error: {}", message);
        self.reset_stack();
    }
    fn reset_stack(&mut self) {
        self.stack.clear();
    }
    fn binary_number_op<T>(&mut self, apply: T) -> bool where T: Fn(f64, f64) -> Value {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        match (a, b) {
            (Value::Number(num_a), Value::Number(num_b)) => {
                self.stack.push(apply(num_a, num_b));
                return true;
            },
            _ => { 
                self.runtime_error("Add operands must be numbers");
                return false;
             }
        }
    } 
    fn is_falsey(&self, val: Value) -> bool {
        return val == Value::Null || val == Value::Bool(false);
    }
}