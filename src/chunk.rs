use crate::{opcode::OpCode, value::Value};

#[derive(PartialEq, Debug, Clone)]
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
    /// Initialises a chunk with only a return operation to terminate vm interpret
    pub fn new_terminated() -> Self {
        let mut chunk = Chunk::new();
        chunk.write_op(OpCode::Return, 1);
        return chunk;
    }
    pub fn write_op(&mut self, op: OpCode, line: usize) {
        self.write_byte(u8::from(op), line);
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