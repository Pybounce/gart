
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ParseRule {
    pub prefix: ParseFn,
    pub infix: ParseFn,
    pub precedence: ParsePrecedence,
}

impl ParseRule {
    pub fn new(prefix: ParseFn, infix: ParseFn, precedence: ParsePrecedence) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ParsePrecedence {
    None,           // Low Precedence
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,        // High Precedence
}

impl TryFrom::<u8> for ParsePrecedence {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ParsePrecedence::None),
            1 => Ok(ParsePrecedence::Assignment),
            2 => Ok(ParsePrecedence::Or),
            3 => Ok(ParsePrecedence::And),
            4 => Ok(ParsePrecedence::Equality),
            5 => Ok(ParsePrecedence::Comparison),
            6 => Ok(ParsePrecedence::Term),
            7 => Ok(ParsePrecedence::Factor),
            8 => Ok(ParsePrecedence::Unary),
            9 => Ok(ParsePrecedence::Call),
            10 => Ok(ParsePrecedence::Primary),
            _ => Err("Failed to convert u8 to ParsePrecedence")
        }
    }
}

impl From::<ParsePrecedence> for u8 {
    fn from(value: ParsePrecedence) -> Self {
        match value {
            ParsePrecedence::None => 0,
            ParsePrecedence::Assignment => 1,
            ParsePrecedence::Or => 2,
            ParsePrecedence::And => 3,
            ParsePrecedence::Equality => 4,
            ParsePrecedence::Comparison => 5,
            ParsePrecedence::Term => 6,
            ParsePrecedence::Factor => 7,
            ParsePrecedence::Unary => 8,
            ParsePrecedence::Call => 9,
            ParsePrecedence::Primary => 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseFn {
    None,   // neater than using option everywhere, sue me. (don't sue me)
    Number,
    Binary,
    Grouping,
    Call,
    Unary,
    Variable,
    String,
    Literal,
    And,
    Or
}
