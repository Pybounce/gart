
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
    Negate,
    Return,
    Null,
    DefineGlobal
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
            OpCode::Return => 12,
            OpCode::Null => 13,
            OpCode::DefineGlobal => 14,
        }
    }
}

impl TryFrom::<u8> for OpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Constant),
            1 => Ok(OpCode::Print),
            2 => Ok(OpCode::Pop),
            3 => Ok(OpCode::Equal),
            4 => Ok(OpCode::Not),
            5 => Ok(OpCode::Greater),
            6 => Ok(OpCode::Less),
            7 => Ok(OpCode::Add),
            8 => Ok(OpCode::Subtract),
            9 => Ok(OpCode::Multiply),
            10 => Ok(OpCode::Divide),
            11 => Ok(OpCode::Negate),
            12 => Ok(OpCode::Return),
            13 => Ok(OpCode::Null),
            14 => Ok(OpCode::DefineGlobal),
            _ => Err("Failed to convert u8 to ParsePrecedence")
        }
    }
}