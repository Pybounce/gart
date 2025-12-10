use crate::{opcode::OpCode, value::Value};


pub struct Chunk {
    ops: Vec<u8>,
    lines: Vec<usize>,
    constants: Vec<Value>
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            ops: vec![],
            lines: vec![],
            constants: vec![]
        }
    }

    pub fn write_op(&mut self, opcode: OpCode, line: usize) {
        self.ops.push(opcode as u8);
        self.lines.push(line);
    }
}