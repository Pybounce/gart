use std::{rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use crate::{chunk::Chunk, compiler::{Compiler, CompilerOutput}, interpreter::RuntimeError, opcode::OpCode, value::{Function, NativeFunction, Value}};


pub struct VM {
    pub stack: Vec<Value>,
    pub globals: Vec<Option<Value>>,
    call_frames: Vec<CallFrame>
}

struct CallFrame {
    function: Rc<Function>, // probably a ref? not sure. Maybe rc.
    /// Index in the stack where this call-frame starts.
    stack_offset: usize,
    pc: usize,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: Vec::new(),
            call_frames: Vec::new()
        }
    }

    pub fn interpret(&mut self, compiler_output: CompilerOutput) -> Result<(), RuntimeError> {
        self.call_frames.push(CallFrame {
            function: Rc::new(compiler_output.script_function),
            stack_offset: 0,
            pc: 0,
        });
        self.globals = vec![None; compiler_output.globals_count];
        for (i, native) in compiler_output.natives.into_iter().enumerate() {
            self.globals[i] = Some(Value::NativeFunc(Rc::new(native)));            
        }
        return self.run();
    }

    fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            let operation = OpCode::try_from(self.read_byte());
            if operation.is_err() { 
                let err = self.runtime_error("Failed to convert byte to opcode");
                return Err(err);
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
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                },
                OpCode::Not => {
                    let val = self.stack.pop().unwrap();
                    let not_val = self.is_falsey(&val);
                    self.stack.push(Value::Bool(not_val));
                },
                OpCode::Greater => { if let Err(e) = self.binary_number_op(|a, b| Value::Bool(a > b)) { return Err(e); } },
                OpCode::Less => { if let Err(e) = self.binary_number_op(|a, b| Value::Bool(a < b)) { return Err(e); } },
                OpCode::Add => {
                    if matches!(&self.stack[self.stack.len() - 1], Value::String(_)) && matches!(&self.stack[self.stack.len() - 2], Value::String(_)) {
                        if let Err(e) = self.concatenate() { return Err(e); }
                    }
                    else if let Err(e) = self.binary_number_op(|a, b| Value::Number(a + b)) { return Err(e); }
                 },
                OpCode::Subtract => { if let Err(e) = self.binary_number_op(|a, b| Value::Number(a - b)) { return Err(e); } },
                OpCode::Multiply => { if let Err(e) = self.binary_number_op(|a, b| Value::Number(a * b)) { return Err(e); } },
                OpCode::Divide => { if let Err(e) = self.binary_number_op(|a, b| Value::Number(a / b)) { return Err(e); } },
                OpCode::Negate => {
                    let val = self.stack.pop().unwrap();
                    if let Value::Number(n) = val {
                        self.stack.push(Value::Number(-n));
                    } else { 
                        let err = self.runtime_error("Negate operand must be number."); 
                        return Err(err);
                    }
                },
                OpCode::Null => self.stack.push(Value::Null),
                OpCode::DefineGlobal => {
                    self.write_global();
                    self.stack.pop();
                },
                OpCode::SetLocal => {
                    let local_stack_index = self.read_byte();
                    let val = self.stack.last().unwrap().clone();
                    self.stack_set(local_stack_index as usize, val);
                },
                OpCode::GetLocal => {
                    let local_stack_index = self.read_byte();
                    let val = self.stack_get(local_stack_index as usize);

                    self.stack.push(val);
                },
                OpCode::SetGlobal => {
                    self.write_global();
                },
                OpCode::GetGlobal => {
                    match self.read_global() {
                        Some(global_val) => { self.stack.push(global_val); },
                        None => {
                            let err = self.runtime_error("Undefined variable.");
                            return Err(err);
                        },
                    }
                },
                OpCode::JumpIfFalse => {
                    let jump = self.read_short() as usize;
                    if self.is_falsey(self.stack.last().unwrap()) {
                        self.frame_mut().pc += jump;
                    }
                },
                OpCode::Jump => {
                    let jump = self.read_short() as usize;
                    self.frame_mut().pc += jump;
                },
                OpCode::JumpBack => {
                    let jump = self.read_short() as usize;
                    self.frame_mut().pc -= jump;
                },
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Call => {
                    let arg_count = self.read_byte() as usize;
                    let callee_idx = self.stack.len() - 1 - arg_count;
                    let callee = self.stack[callee_idx].clone();
                    if self.call_value(callee, arg_count) == false {
                        let err = self.runtime_error("Failed to call callee"); //TODO - trash error
                        return Err(err);
                    }
                },
                OpCode::Return => {
                    let return_val = self.stack.pop().unwrap();
                    self.stack.truncate(self.frame().stack_offset);
                    self.call_frames.pop();

                    if self.call_frames.len() == 0 {
                        self.stack.pop();
                        return Ok(());
                    }

                    self.stack.push(return_val);
                },
            }
        }
    }
}


// Helpers
impl VM {
    fn pc(&self) -> usize {
        return self.frame().pc;
    }

    fn chunk(&self) -> &Chunk {
        return &self.frame().function.chunk;
    }

    fn frame(&self) -> &CallFrame {
        return self.call_frames.last().unwrap();
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        return self.call_frames.last_mut().unwrap();
    }

    fn stack_set(&mut self, index: usize, val: Value) {
        let stack_offset = self.frame().stack_offset;
        return self.stack[index + stack_offset] = val;
    }

    fn stack_get(&self, index: usize) -> Value {
        let stack_offset = self.frame().stack_offset;
        return self.stack[index + stack_offset].clone();
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk().bytes[self.pc()];
        self.frame_mut().pc += 1;
        return byte;
    }
    fn read_short(&mut self) -> u16 {
        self.frame_mut().pc += 2;
        let high = self.chunk().bytes[self.pc() - 2] as u16;
        let low = self.chunk().bytes[self.pc() - 1] as u16;
        return (high << 8) | low;

    }
    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        return self.chunk().constants[index].clone();
    }
    fn read_global(&mut self) -> Option<Value> {
        let index = self.read_byte() as usize;
        return self.globals[index].clone();
    }
    fn write_global(&mut self) {
        let val = self.stack.last().unwrap().clone();
        let index = self.read_byte() as usize;
        self.globals[index] = Some(val);
    }
    fn runtime_error(&mut self, message: &'static str) -> RuntimeError {
        println!("Runtime error: {}", message);
        self.reset_stack();
        return RuntimeError {
            message
        };
    }
    fn reset_stack(&mut self) {
        self.stack.clear();
    }
    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        match callee {
            Value::Bool(_)
            | Value::Number(_)
            | Value::Null
            | Value::String(_) => {
                return false;
            },
            Value::Func(function) => {
                if arg_count != function.arity as usize {
                    //TODO: Allow formatted strings in errors self.runtime_error("Expected {} arguments, but got {}", function.arity, arg_count);
                    self.runtime_error("Incorrect argument count.");
                    return false;
                }

                let frame = CallFrame {
                    function,
                    stack_offset: self.stack.len() - 1 - arg_count,
                    pc: 0,
                };
                self.call_frames.push(frame);
            },
            Value::NativeFunc(native_function) => {
                let args_slice = &self.stack[(self.stack.len() - arg_count)..self.stack.len()];
                let return_val = (native_function.function)(args_slice);
                self.stack.truncate(self.stack.len() - arg_count);
                self.stack.push(return_val);
            },
        }

        return true;
    }
    fn binary_number_op<T>(&mut self, apply: T) -> Result<(), RuntimeError> where T: Fn(f64, f64) -> Value {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        match (a, b) {
            (Value::Number(num_a), Value::Number(num_b)) => {
                self.stack.push(apply(num_a, num_b));
                return Ok(());
            },
            _ => { 
                let err = self.runtime_error("Add operands must both be strings or numbers");
                return Err(err);
             }
        }
    } 

    fn concatenate(&mut self) -> Result<(), RuntimeError> {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        match (a, b) {
            (Value::String(str_a), Value::String(str_b)) => {
                self.stack.push(Value::String(Rc::new(str_a.as_str().to_owned() + str_b.as_str())));
                return Ok(());
            },
            _ => { 
                let err = self.runtime_error("Add operands must both be strings or numbers");
                return Err(err);
             }
        }    
    }

    fn is_falsey(&self, val: &Value) -> bool {
        return *val == Value::Null || *val == Value::Bool(false);
    }
}