
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token {
    pub token_type: TokenType,
    /// Index of first lexeme character in source
    pub start: usize,
    /// Length of lexeme
    pub length: usize,
    pub line: usize
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, length: usize, line: usize) -> Self {
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
    Return,
    True,
    Var,
    While,
    Error,
    Eof
}