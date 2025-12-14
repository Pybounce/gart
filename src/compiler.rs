use std::collections::HashMap;

use crate::{chunk::Chunk, opcode::OpCode, parse::{ParseFn, ParsePrecedence, ParseRule}, scanner::Scanner, token::{Token, TokenType}, value::Value};


pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    source: &'a str,
    chunk: Chunk,
    previous_token: Token,
    current_token: Token,
    had_error: bool,
    panic_mode: bool,
    globals_state: HashMap<&'a str, (u8, bool, Vec<Token>)>
}

#[derive(PartialEq, Debug)]
pub struct CompilerOutput {
    pub chunk: Chunk,
    pub globals_count: usize
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
            globals_state: HashMap::new()
        }
    }
    pub fn compile(mut self) -> Option<CompilerOutput> {
        self.advance();
        while self.match_token(TokenType::Eof) == false {
            self.declaration();
        }

        self.finish();

        return (!self.had_error).then(|| CompilerOutput { chunk: self.chunk, globals_count: self.globals_state.len() });
    }

    fn finish(&mut self) {
        self.emit_op(OpCode::Return);
    }
}



// Statements/Declarations/Expressions
impl<'a> Compiler<'a> {
    fn declaration(&mut self) {
        if self.match_token(TokenType::Fn) { todo!(); }
        else if self.match_token(TokenType::Var) { self.var_declaration(); }
        else { self.statement(); }

        if self.panic_mode { self.synchronise(); }
    }

    fn var_declaration(&mut self) {
        let global_index = self.parse_variable("Expect variable name.", true);
        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_op(OpCode::Null);
        }
        self.consume(TokenType::NewLine, "Expect newline after expression.");
        self.emit_op(OpCode::DefineGlobal);
        self.emit_byte(global_index);
    }

    fn parse_variable(&mut self, error_msg: &'static str, is_declaration: bool) -> u8 {
        self.consume(TokenType::Identifier, error_msg);
        return self.global_identifier(self.previous_token, is_declaration);
    }

    /// Gets the globals index for the identifier. </br>
    /// If identifier does not exist in globals, it will add it and return index. </br>
    fn global_identifier(&mut self, token: Token, is_declaration: bool) -> u8 {
        let identifier_name = &self.source[token.start..(token.start + token.length)];
        if let Some((index, declared, tokens_using)) = self.globals_state.get_mut(identifier_name) {
            if is_declaration { *declared = true; }
            else { tokens_using.push(token); }
            return *index;
        } else {
            let globals_count = self.globals_state.len() as u8;
            if globals_count == u8::MAX {
                self.error_at_previous("Too many globals.");
                return 0;
            }
            self.globals_state.insert(identifier_name, (globals_count, is_declaration, vec![token]));
            return globals_count;
        }
    }

    fn variable(&mut self, can_assign: bool) {
        let identifier_token = self.previous_token;
        let (get_op, set_op, index): (OpCode, OpCode, u8) = match self.local_index() {
            Some(local_index) => (OpCode::GetLocal, OpCode::SetLocal, local_index),
            None => (OpCode::GetGlobal, OpCode::SetGlobal, self.global_identifier(identifier_token, false)),
        };

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_op(set_op);
            self.emit_byte(index);
        }
        else {
            self.emit_op(get_op);
            self.emit_byte(index);
        }
    }

    // Tries to find local, returns index if it can. </br>
    // Returns none otherwise.
    fn local_index(&self) -> Option<u8> {
        return None;
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
        self.consume(TokenType::NewLine, "Expect newline after expression.");
        self.emit_op(OpCode::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::NewLine, "Expect newline after expression.");
        self.emit_op(OpCode::Pop);
    }

    fn expression(&mut self) {
        self.parse_precedence(ParsePrecedence::Assignment);
    }

    fn number(&mut self, can_assign: bool) {
        let lexeme = &self.source[self.previous_token.start..self.previous_token.length + self.previous_token.start];
        if let Ok(number) = lexeme.parse::<f64>() {
            self.emit_constant(Value::Number(number));
        }
        else {
            self.error_at_previous("Failed to parse number.");
        }
    }

    fn binary(&mut self, can_assign: bool) {
        let operator = self.previous_token.token_type;
        let operator_rule_prec = self.get_rule(operator).precedence;
        match ParsePrecedence::try_from(u8::from(operator_rule_prec) + 1) {
            Ok(new_precedence) => self.parse_precedence(new_precedence),
            Err(msg) => self.error_at_current(msg),
        }
        
        match operator {
            TokenType::BangEqual =>     self.emit_ops(OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual =>    self.emit_op(OpCode::Equal),
            TokenType::Greater =>       self.emit_op(OpCode::Greater),
            TokenType::GreaterEqual =>  self.emit_ops(OpCode::Less, OpCode::Not),
            TokenType::Less =>          self.emit_op(OpCode::Less),
            TokenType::LessEqual =>     self.emit_ops(OpCode::Greater, OpCode::Not),
            TokenType::Plus =>          self.emit_op(OpCode::Add),
            TokenType::Minus =>         self.emit_op(OpCode::Subtract),
            TokenType::Star =>          self.emit_op(OpCode::Multiply),
            TokenType::Slash =>         self.emit_op(OpCode::Divide),
            _ => self.error_at_current("binary operator mismatch."),
        };
        
    }

    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, can_assign: bool) {
        let operator = self.previous_token.token_type;
        self.parse_precedence(ParsePrecedence::Unary);

        match operator {
            TokenType::Bang => self.emit_op(OpCode::Not),
            TokenType::Minus => self.emit_op(OpCode::Negate),
            _ => self.error_at_previous("Unreachable unary operator...reached."),
        }
    }

    fn parse_precedence(&mut self, precedence: ParsePrecedence) {
        self.advance();
        let prefix_fn = self.get_rule(self.previous_token.token_type).prefix;
        if prefix_fn == ParseFn::None { 
            self.error_at_current("Expected expression.");
            return;
        }

        let can_assign = precedence <= ParsePrecedence::Assignment;
        self.call_parse_fn(prefix_fn, can_assign);

        while precedence <= self.get_rule(self.current_token.token_type).precedence {
            self.advance();
            let infix_fn = self.get_rule(self.previous_token.token_type).infix;
            self.call_parse_fn(infix_fn, can_assign);
        }

        if can_assign && self.match_token(TokenType::Equal) { self.error_at_current("Invalid assignment target."); }
    }

    fn call_parse_fn(&mut self, parse_fn: ParseFn, can_assign: bool) {
        match parse_fn {
            ParseFn::None => (),
            ParseFn::Number => self.number(can_assign),
            ParseFn::Binary => self.binary(can_assign),
            ParseFn::Grouping => self.grouping(can_assign),
            ParseFn::Call => todo!(),
            ParseFn::Unary => self.unary(can_assign),
            ParseFn::Variable => self.variable(can_assign),
            ParseFn::String => todo!(),
            ParseFn::Literal => todo!(),
            ParseFn::And => todo!(),
            ParseFn::Or => todo!(),
        };
    }

    fn get_rule(&self, token_type: TokenType) -> ParseRule {
        match token_type {
            TokenType::LeftParen =>     ParseRule::new(ParseFn::Grouping, ParseFn::Call, ParsePrecedence::Call),
            TokenType::RightParen =>    ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Indent =>        ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Dedent =>        ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::NewLine =>       ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Comma =>         ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Minus =>         ParseRule::new(ParseFn::Unary, ParseFn::Binary, ParsePrecedence::Term),
            TokenType::Plus =>          ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Term),
            TokenType::Colon =>         ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Slash =>         ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Factor),
            TokenType::Star =>          ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Factor),
            TokenType::Bang =>          ParseRule::new(ParseFn::Unary, ParseFn::None, ParsePrecedence::None),
            TokenType::BangEqual =>     ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Equality),
            TokenType::Equal =>         ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::EqualEqual =>    ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Equality),
            TokenType::Greater =>       ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Comparison),
            TokenType::GreaterEqual =>  ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Comparison),
            TokenType::Less =>          ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Comparison),
            TokenType::LessEqual =>     ParseRule::new(ParseFn::None, ParseFn::Binary, ParsePrecedence::Comparison),
            TokenType::Identifier =>    ParseRule::new(ParseFn::Variable, ParseFn::None, ParsePrecedence::None),
            TokenType::String =>        ParseRule::new(ParseFn::String, ParseFn::None, ParsePrecedence::None),
            TokenType::Number =>        ParseRule::new(ParseFn::Number, ParseFn::None, ParsePrecedence::None),
            TokenType::And =>           ParseRule::new(ParseFn::None, ParseFn::And, ParsePrecedence::And),
            TokenType::Else =>          ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::False =>         ParseRule::new(ParseFn::Literal, ParseFn::None, ParsePrecedence::None),
            TokenType::For =>           ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Fn =>            ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::If =>            ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Null =>          ParseRule::new(ParseFn::Literal, ParseFn::None, ParsePrecedence::None),
            TokenType::Or =>            ParseRule::new(ParseFn::None, ParseFn::Or, ParsePrecedence::Or),
            TokenType::Print =>         ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Return =>        ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::True =>          ParseRule::new(ParseFn::Literal, ParseFn::None, ParsePrecedence::None),
            TokenType::Var =>           ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::While =>         ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Error =>         ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
            TokenType::Eof =>           ParseRule::new(ParseFn::None, ParseFn::None, ParsePrecedence::None),
        }
    }

}

impl<'a> Compiler<'a> {
    fn emit_op(&mut self, code: OpCode) {
        self.emit_byte(code as u8);
    }

    fn emit_ops(&mut self, op1: OpCode, op2: OpCode) {
        self.emit_op(op1);
        self.emit_op(op2);
    }
    
    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write_byte(byte, self.previous_token.line);
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_op(OpCode::Constant);
        let constant_index = self.make_constant(value);
        self.emit_byte(constant_index);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant_index = self.chunk.write_constant(value);
        if let Ok(index_u8) = u8::try_from(constant_index) {
            return index_u8;
        }
        else {
            self.error_at_current("Too many constants in one chunk. Max 256.");
            return 0;
        }
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

#[cfg(test)]
mod test {
    use crate::{chunk::Chunk, compiler::Compiler, opcode::OpCode, value::Value};

    #[test]
    fn print_number() {
        let source = r#"print 1"#;
        let compiler = Compiler::new(&source);

        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::Print.into(),
                OpCode::Return.into()
            ],
            lines: vec![1, 1, 1, 1],
            constants: vec![Value::Number(1.0)],
        };
        
        let output = compiler.compile();
        assert_eq!(expected_chunk, output.expect("Failed to compile").chunk);
    }

    #[test]
    fn arithmetic() {
        let source = r#"1 + 2 * (5 - 3)"#;
        let compiler = Compiler::new(&source);

        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::Constant.into(),
                1,
                OpCode::Constant.into(),
                2,
                OpCode::Constant.into(),
                3,
                OpCode::Subtract.into(),
                OpCode::Multiply.into(),
                OpCode::Add.into(),
                OpCode::Pop.into(),
                OpCode::Return.into()
            ],
            lines: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            constants: vec![
                Value::Number(1.0), 
                Value::Number(2.0), 
                Value::Number(5.0), 
                Value::Number(3.0)
            ]
        };
        
        let output = compiler.compile();
        assert_eq!(expected_chunk, output.expect("Failed to compile").chunk);
    }

    #[test]
    fn error_trailing_arithmetic_op() {
        let source = r#"1 +"#;
        let compiler = Compiler::new(&source);
        
        let output = compiler.compile();
        assert_eq!(None, output);
    }

    #[test]
    fn arithmetic_minus_unary() {
        let source = r#"-10.4"#;
        let compiler = Compiler::new(&source);

        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::Negate.into(),
                OpCode::Pop.into(),
                OpCode::Return.into()
            ],
            lines: vec![1, 1, 1, 1, 1],
            constants: vec![Value::Number(10.4)],
        };

        let output = compiler.compile();
        assert_eq!(expected_chunk, output.expect("Failed to compile").chunk);
    }

    #[test]
    fn single_line() {
        let source = r#"print 1"#;
        let compiler = Compiler::new(&source);

        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::Print.into(),
                OpCode::Return.into()
            ],
            lines: vec![1, 1, 1, 1],
            constants: vec![Value::Number(1.0)],
        };
        
        let output = compiler.compile();
        assert_eq!(expected_chunk, output.expect("Failed to compile").chunk);
    }

    #[test]
    fn newline_start() {
        let source = r#"    

print 1"#;
        let compiler = Compiler::new(&source);

        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::Print.into(),
                OpCode::Return.into()
            ],
            lines: vec![3, 3, 3, 3],
            constants: vec![Value::Number(1.0)],
        };
        
        let output = compiler.compile();
        assert_eq!(expected_chunk, output.expect("Failed to compile").chunk);
    }

    #[test]
    fn newline_end() {
        let source = r#"print 1
"#;
        let compiler = Compiler::new(&source);

        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::Print.into(),
                OpCode::Return.into()
            ],
            lines: vec![1, 1, 1, 2],
            constants: vec![Value::Number(1.0)],
        };
        
        let output = compiler.compile();
        assert_eq!(expected_chunk, output.expect("Failed to compile").chunk);
    }

    #[test]
    fn global_declarations() {
        let source = r#"
var g = 1
var g2 = 2"#;
        let compiler = Compiler::new(&source);

        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::DefineGlobal.into(),
                0,
                OpCode::Constant.into(),
                1,
                OpCode::DefineGlobal.into(),
                1,
                OpCode::Return.into()
            ],
            lines: vec![2, 2, 2, 2, 3, 3, 3, 3, 3],
            constants: vec![Value::Number(1.0), Value::Number(2.0)],
        };
        let expected_global_count = 2;
        
        let output = compiler.compile().expect("Failed to compile");
        assert_eq!(expected_chunk, output.chunk);
        assert_eq!(expected_global_count, output.globals_count);
    }

    #[test]
    fn global_assignment() {
        let source = r#"
var g = 1
var g2 = 2
g = 4"#;
        let compiler = Compiler::new(&source);

        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::DefineGlobal.into(),
                0,
                OpCode::Constant.into(),
                1,
                OpCode::DefineGlobal.into(),
                1,
                OpCode::Constant.into(),
                2,
                OpCode::SetGlobal.into(),
                0,
                OpCode::Pop.into(),
                OpCode::Return.into()
            ],
            lines: vec![2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4],
            constants: vec![Value::Number(1.0), Value::Number(2.0), Value::Number(4.0)],
        };
        let expected_global_count = 2;
        
        let output = compiler.compile().expect("Failed to compile");
        assert_eq!(expected_chunk, output.chunk);
        assert_eq!(expected_global_count, output.globals_count);
    }

    #[test]
    fn error_redeclaration() {
        let source = r#"
var g = 1
var g = 2"#;
        let compiler = Compiler::new(&source);
        
        let output = compiler.compile();
        assert_eq!(output, None);
    }

    #[test]
    /// For now this remains a runtime error
    fn undefined_variable() {
        let source = r#"g = 1"#;
        let compiler = Compiler::new(&source);
        
        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::SetGlobal.into(),
                0,
                OpCode::Pop.into(),
                OpCode::Return.into()
            ],
            lines: vec![1, 1, 1, 1, 1, 1],
            constants: vec![Value::Number(1.0)],
        };
        let expected_global_count = 1;

        let output = compiler.compile().expect("Failed to compile");
        assert_eq!(expected_chunk, output.chunk);
        assert_eq!(expected_global_count, output.globals_count);
    }

    #[test]
    fn chained_assignment() {
        let source = r#"
var a = 1
var b
var c = b = a"#;
        let compiler = Compiler::new(&source);
        
        let expected_chunk = Chunk {
            bytes: vec![
                OpCode::Constant.into(),
                0,
                OpCode::DefineGlobal.into(),
                0,
                OpCode::Null.into(),
                OpCode::DefineGlobal.into(),
                1,
                OpCode::GetGlobal.into(),
                0,
                OpCode::SetGlobal.into(),
                1,
                OpCode::DefineGlobal.into(),
                2,
                OpCode::Return.into()
            ],
            lines: vec![2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4],
            constants: vec![Value::Number(1.0)],
        };
        let expected_global_count = 3;

        let output = compiler.compile().expect("Failed to compile");
        assert_eq!(expected_chunk, output.chunk);
        assert_eq!(expected_global_count, output.globals_count);
    }
}