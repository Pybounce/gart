
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token {
    pub token_type: TokenType,
    /// Index of first lexeme character in source
    pub start: i32,
    /// Length of lexeme
    pub length: i32,
    pub line: i32
}

impl Token {
    pub fn new(token_type: TokenType, start: i32, length: i32, line: i32) -> Self {
        Self {
            token_type,
            start,
            length,
            line,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenType {
    LeftParen,
    RightParen,
    Indent,
    Dedent,
    NewLine,
    Comma,
    Minus,
    Plus,
    Colon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Else,
    False,
    For,
    Fn,
    If,
    Null,
    Or,
    Print,
    Return,
    True,
    Var,
    While,
    Error,
    Eof
}