use std::fmt::Display;


#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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
    DefineGlobal,
    SetLocal,
    GetLocal,
    SetGlobal,
    GetGlobal,
    JumpIfFalse,
    Jump,
    JumpBack,
    True,
    False,
    Call,
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
            OpCode::SetLocal => 15,
            OpCode::GetLocal => 16,
            OpCode::SetGlobal => 17,
            OpCode::GetGlobal => 18,
            OpCode::JumpIfFalse => 19,
            OpCode::Jump => 20,
            OpCode::JumpBack => 21,
            OpCode::True => 22,
            OpCode::False => 23,
            OpCode::Call => 24,
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
            15 => Ok(OpCode::SetLocal),
            16 => Ok(OpCode::GetLocal),
            17 => Ok(OpCode::SetGlobal),
            18 => Ok(OpCode::GetGlobal),
            19 => Ok(OpCode::JumpIfFalse),
            20 => Ok(OpCode::Jump),
            21 => Ok(OpCode::JumpBack),
            22 => Ok(OpCode::True),
            23 => Ok(OpCode::False),
            24 => Ok(OpCode::Call),
            _ => Err("Failed to convert u8 to ParsePrecedence")
        }
    }
}