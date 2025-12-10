use crate::{chunk::Chunk, opcode::OpCode, scanner::Scanner, token::{Token, TokenType}};


pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    source: &'a str,
    chunk: Chunk,
    previous_token: Token,
    current_token: Token,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            source: source,
            chunk: Chunk::new(),
            previous_token: Token::new(TokenType::Error, 0, 0, 0),  // I know.
            current_token: Token::new(TokenType::Error, 0, 0, 0),
            had_error: false,
            panic_mode: false,
        }
    }
    pub fn compile(mut self) -> Chunk {
        
        while self.match_token(TokenType::Eof) == false {
            self.declaration();
        }
        return self.chunk;
    }
}

// Statements/Declarations/Expressions
impl<'a> Compiler<'a> {
    fn declaration(&mut self) {
        if self.match_token(TokenType::Fn) { todo!(); }
        else if self.match_token(TokenType::Var) { todo!(); }
        else { self.statement(); }

        if self.panic_mode { self.synchronise(); }
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        }
        else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::NewLine, "Expect newline after expression."); //TODO: This probably breaks if instead you have EoF
        self.emit_byte(OpCode::Print);
    }

    fn expression_statement(&mut self) {

    }

    fn expression(&mut self) {

    }
}

impl<'a> Compiler<'a> {
    fn emit_byte(&mut self, code: OpCode) {
        self.chunk.write_op(code, self.previous_token.line);
    }
}

// Helpers
impl<'a> Compiler<'a> {

    fn synchronise(&mut self) {
        self.panic_mode = false;

        while self.current_token.token_type != TokenType::Eof {
            //TODO: Need to solve for end of expression, newline is not great.
            if self.previous_token.token_type == TokenType::NewLine { return; }
            match self.current_token.token_type {
                TokenType::Fn
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => { }
            }
            self.advance();
        }

    }

    fn advance(&mut self) {
        self.previous_token = self.current_token;
        loop {
            self.current_token = self.scanner.scan_token();
            if self.current_token.token_type != TokenType::Error { break; }
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &'static str) {
        if self.current_token.token_type == token_type {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check_token(token_type) == false { return false; }
        self.advance();
        return true;
    }

    fn check_token(&self, token_type: TokenType) -> bool {
        return self.current_token.token_type == token_type;
    }

    fn error_at_current(&mut self, message: &'static str) {
        self.error_at(self.current_token, message);
    }

    fn error_at(&mut self, token: Token, message: &'static str) {
        if self.panic_mode { return; }
        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        match token.token_type {
            TokenType::Eof => { eprint!(" at end"); },
            TokenType::Error => { }
            _ => { eprint!(" at {}", &self.source[token.start..(token.start + token.length)]); }
        }

        eprint!(": {}\n", message);
        self.had_error = true;
    }

    fn error_at_previous(&mut self, message: &'static str) {
        self.error_at(self.previous_token, message);
    }
}