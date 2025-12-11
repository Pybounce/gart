use crate::{opcode::OpCode, value::Value};

#[derive(PartialEq, Debug)]
pub struct Chunk {
    pub bytes: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Value>
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            lines: vec![],
            constants: vec![]
        }
    }
    pub fn write_op(&mut self, op: OpCode, line: usize) {
        self.write_byte(op as u8, line);
    }
    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.bytes.push(byte);
        self.lines.push(line);
    }
    pub fn write_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        return self.constants.len() - 1;
    }
}