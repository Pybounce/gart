
#[repr(u8)]
pub enum OpCode {
    Constant,
    Print,
    Pop,
    Equal,
    Not,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate
}

impl From::<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Constant => 0,
            OpCode::Print => 1,
            OpCode::Pop => 2,
            OpCode::Equal => 3,
            OpCode::Not => 4,
            OpCode::Greater => 5,
            OpCode::Less => 6,
            OpCode::Add => 7,
            OpCode::Subtract => 8,
            OpCode::Multiply => 9,
            OpCode::Divide => 10,
            OpCode::Negate => 11,
        }
    }
}
