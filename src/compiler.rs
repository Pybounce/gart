use std::{backtrace, collections::HashMap, rc::Rc};

use crate::{chunk::Chunk, interpreter::CompilerError, opcode::OpCode, parse::{ParseFn, ParsePrecedence, ParseRule}, scanner::Scanner, token::{Token, TokenType}, value::{Function, NativeFunction, Value}};


pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    source: &'a str,
    previous_token: Token,
    current_token: Token,
    had_error: bool,
    panic_mode: bool,
    errors: Vec<CompilerError>,
    globals_state: HashMap<String, (u8, bool, Vec<Token>)>,
    funpiler_stack: Vec<Funpiler>,
    natives: Vec<NativeFunction>,

}

struct Funpiler {
    locals: Vec<Local>,
    scope_depth: usize,
    chunk: Chunk,
    arity: u8,
    name: String
}

impl Funpiler {
    pub fn new() -> Self {
        return Self {
            locals: vec![],
            scope_depth: 0,
            chunk: Chunk::new(),
            arity: 0,
            name: "".to_owned()
        };
    }
}

#[derive(Clone, Copy)]
struct Local {
    token: Token,
    depth: i32
}

#[derive(PartialEq, Debug)]
pub struct CompilerOutput {
    pub script_function: Function,
    pub globals_count: usize,
    pub natives: Vec<NativeFunction>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            source: source,
            previous_token: Token::new(TokenType::Error, 0, 0, 0),  // I know.
            current_token: Token::new(TokenType::Error, 0, 0, 0),
            had_error: false,
            panic_mode: false,
            errors: vec![],
            globals_state: HashMap::new(),
            natives: vec![],
            funpiler_stack: vec![]
        }
    }
    pub fn add_native(&mut self, native: NativeFunction) {
        let index = self.insert_global(native.name.to_owned(), true, None, true);
        if self.natives.len() == index as usize {
            self.natives.push(native);
        }
        else if self.natives.len() > index as usize {
            self.natives[index as usize] = native;
        }
        else {
            self.error_at_current("Failed to add native function. Index error.");
        }
    }
    pub fn compile(mut self) -> Result<CompilerOutput, Vec<CompilerError>> {
        self.new_funpiler();
        self.advance();
        while self.match_token(TokenType::Eof) == false {
            self.declaration();
        }

        let script_function = self.end_funpiler();


        let globals: Vec<_> = self.globals_state.values().cloned().collect();
        for (_, declared, tokens) in globals {
            if !declared {
                for token in tokens.iter() {
                    self.error_at(*token, "Undefined global variable.");
                    self.panic_mode = false;
                }
            }
        }

        if self.had_error {
            return Err(self.errors)
        }
        return Ok(CompilerOutput { script_function, globals_count: self.globals_state.len(), natives: self.natives });
    }
}



// Statements/Declarations/Expressions
impl<'a> Compiler<'a> {
    fn declaration(&mut self) {
        if self.match_token(TokenType::Fn) { self.fn_declaration(); }
        else if self.match_token(TokenType::Var) { self.var_declaration(); }
        else { self.statement(); }

        if self.panic_mode { self.synchronise(); }
    }

    fn fn_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect function name.");
        if self.funpiler().scope_depth > 0 { 
            self.error_at_previous("Cannot declare function inside another function.");
        }
        let global_index = self.global_identifier(self.previous_token, true);

        let function = self.function();

        self.emit_constant(Value::Func(Rc::new(function)));

        self.emit_op(OpCode::DefineGlobal);
        self.emit_byte(global_index);
    }

    fn new_funpiler(&mut self) {
        self.funpiler_stack.push(Funpiler::new());
        self.funpiler().locals.push(Local {
            token: Token::new(TokenType::Null, 0, 0, 0),
            depth: 0,
        });
    }

    fn end_funpiler(&mut self) -> Function {
        self.emit_byte(OpCode::Null);
        self.emit_byte(OpCode::Return);
        let funpiler = self.funpiler_stack.pop().unwrap();
        let function = Function {
            name: funpiler.name,
            arity: funpiler.arity,
            chunk: funpiler.chunk,
        };
        return function;
    }

    fn function(&mut self) -> Function {
        self.new_funpiler();
        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.");
        if !self.check_token(TokenType::RightParen) {
            loop {
                if self.funpiler().arity < u8::MAX {
                    self.funpiler().arity += 1;
                    
                    self.consume(TokenType::Identifier, "Expect parameter name.");
                    let new_local = self.previous_token;

                    for i in (0..self.funpiler().locals.len()).rev() {
                        let local = self.funpiler().locals[i];
                        if local.depth != -1 && local.depth < self.funpiler().scope_depth as i32 { break; }
                    
                        if self.identifiers_equal(local.token, new_local) {
                            self.error_at_current("Already a variable with this name in scope.");
                            break;
                        }
                    }
                
                    if self.funpiler().locals.len() == u8::MAX as usize{
                        self.error_at_current("Local variable count has been exceeded.");
                    }
                    let depth = self.funpiler().scope_depth;
                    self.funpiler().locals.push(Local { token: new_local, depth: depth as i32});

                    if !self.match_token(TokenType::Comma) { break; }
                }
                else {
                    self.error_at_current("Cannot have more than 255 parameters.");
                }
                
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::Colon, "Expect ':' after function definition.");
        self.consume(TokenType::NewLine, "Expect newline after ':' in function definition.");
        self.consume(TokenType::Indent, "Expect indentation.");
        self.block();

        return self.end_funpiler();
        
    }

    fn arguments(&mut self) -> u8 {
        let mut arg_count: u8 = 0;
        if !self.check_token(TokenType::RightParen) {
            loop {
                if arg_count == u8::MAX {
                    self.error_at_current("Cannot have more than 255 arguments");
                    break;
                }
                self.expression();
                arg_count += 1;
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        return arg_count;
    }

    fn call(&mut self) {
        let arg_count = self.arguments();
        self.emit_bytes(OpCode::Call, arg_count);
    }

    fn var_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect variable name.");
        if self.funpiler().scope_depth == 0 {
            self.var_global();
        }
        else {
            self.var_local();
        }
    }

    fn var_local(&mut self) {
        let new_local = self.previous_token;

        for i in (0..self.funpiler().locals.len()).rev() {
            let local = self.funpiler().locals[i];
            if local.depth != -1 && local.depth < self.funpiler().scope_depth as i32 { break; }

            if self.identifiers_equal(local.token, new_local) {
                self.error_at_current("Already a variable with this name in scope.");
                break;
            }
        }

        if self.funpiler().locals.len() == u8::MAX as usize{
            self.error_at_current("Local variable count has been exceeded.");
        }
        self.funpiler().locals.push(Local { token: new_local, depth: -1 });

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_op(OpCode::Null);
        }
        self.consume(TokenType::NewLine, "Expect newline after expression.");

        let funpiler = self.funpiler();
        if let Some(local) = funpiler.locals.last_mut() {
            local.depth = funpiler.scope_depth as i32;
        }
    }

    fn var_global(&mut self) {
        let global_index = self.global_identifier(self.previous_token, true);

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_op(OpCode::Null);
        }
        self.consume(TokenType::NewLine, "Expect newline after expression.");
        self.emit_op(OpCode::DefineGlobal);
        self.emit_byte(global_index);
    }

    fn identifiers_equal(&self, a: Token, b: Token) -> bool {
        if a.length != b.length { return false; }
        let a_str = &self.source[a.start..(a.start + a.length)];
        let b_str = &self.source[b.start..(b.start + b.length)];
        return a_str == b_str;
    }

    /// Gets the globals index for the identifier. </br>
    /// If identifier does not exist in globals, it will add it and return index. </br>
    fn global_identifier(&mut self, token: Token, is_declaration: bool) -> u8 {
        let identifier_name = &self.source[token.start..(token.start + token.length)];
        return self.insert_global(identifier_name.to_owned(), is_declaration, token.into(), false);
    }

    fn insert_global(&mut self, name: String, is_declaration: bool, token: Option<Token>, overwrite: bool) -> u8 {
        if let Some((index, declared, tokens_using)) = self.globals_state.get_mut(&name) {
            if *declared && is_declaration && !overwrite && token.is_some() {
                self.error_at(token.unwrap(), "Aready a global variable with this name.");
                return 0;
            }
            if is_declaration { *declared = true; }
            else { 
                if let Some(token) = token {
                    tokens_using.push(token);
                }
            }
            return *index;
        } else {
            let globals_count = self.globals_state.len() as u8;
            if globals_count == u8::MAX {
                self.error_at_previous("Too many globals.");
                return 0;
            }
            let tokens = if let Some(token) = token { vec![token] } else { vec![] };
            self.globals_state.insert(name, (globals_count, is_declaration, tokens));
            return globals_count;
        }
    }

    fn variable(&mut self, can_assign: bool) {
        let identifier_token = self.previous_token;
        let (get_op, set_op, index): (OpCode, OpCode, u8) = match self.local_index(identifier_token) {
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
    fn local_index(&mut self, identifier_token: Token) -> Option<u8> {
        for i in (0..self.funpiler().locals.len()).rev() {
            let local = self.funpiler().locals[i];
            if self.identifiers_equal(local.token, identifier_token) {
                if local.depth == -1 { 
                    self.error_at_current("Can't read local variable in it's own initialiser.");
                }
                return Some(i as u8);
            }
        }
        return None;
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        }
        else if self.match_token(TokenType::If) {
            self.if_statement();
        }
        else if self.match_token(TokenType::Return) {
            self.return_statement();
        }
        else if self.match_token(TokenType::While) {
            self.while_statement();
        }
        else if self.match_token(TokenType::Indent) {
            self.begin_scope();
            self.block();
            self.end_scope();
        }
        else {
            self.expression_statement();
        }
    }

    fn return_statement(&mut self) {
        if self.funpiler_stack.len() <= 1 {
            self.error_at_current("Cannot return from top-level code.");
        }

        if self.match_token(TokenType::NewLine) {
            self.emit_bytes(OpCode::Null, OpCode::Return);
        }
        else {
            self.expression();
            self.consume(TokenType::NewLine, "Expect newline after return value.");
            self.emit_byte(OpCode::Return);
        }
    }

    fn if_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Colon, "Expect ':' after condition.");
        self.consume(TokenType::NewLine, "Expect newline after ':'");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();
    
        let else_jump = self.emit_jump(OpCode::Jump);
    
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop);
    
        if self.match_token(TokenType::Else) {
            if !self.check_token(TokenType::If) { 
                self.consume(TokenType::Colon, "Expect ':' after 'else'.");
                self.consume(TokenType::NewLine, "Expect newline after ':'");
             }
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let jump_landing = self.funpiler().chunk.bytes.len();
        self.expression();
        self.consume(TokenType::Colon, "Expect ':' after condition.");
        self.consume(TokenType::NewLine, "Expect newline after ':'");
        let loop_break_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_back_jump(jump_landing);

        self.patch_jump(loop_break_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn block(&mut self) {
        while !self.check_token(TokenType::Dedent) && !self.check_token(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::Dedent, "Expect dedent after block.");
    }

    fn begin_scope(&mut self) {
        self.funpiler().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.funpiler().scope_depth -= 1;
        while let Some(local) = self.funpiler().locals.last() {
            if local.depth <= self.funpiler().scope_depth as i32 { break; }
            self.funpiler().locals.pop();
            self.emit_byte(OpCode::Pop);
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

    fn string(&mut self, can_assign: bool) {
        let val = &self.source[(self.previous_token.start + 1)..(self.previous_token.length + self.previous_token.start - 1)];
        self.emit_constant(Value::String(Rc::new(val.to_owned())));
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

    fn and(&mut self, can_assign: bool) {
        let jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(ParsePrecedence::And);
        self.patch_jump(jump);
    }

    fn or(&mut self, can_assign: bool) {
        let hop = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(hop);

        self.emit_byte(OpCode::Pop);
        self.parse_precedence(ParsePrecedence::Or);
        self.patch_jump(end_jump);
    }

    fn literal(&mut self, can_assign: bool) {
        match self.previous_token.token_type {
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Null => self.emit_byte(OpCode::Null),
            _ => return
        }
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
            ParseFn::Call => self.call(),
            ParseFn::Unary => self.unary(can_assign),
            ParseFn::Variable => self.variable(can_assign),
            ParseFn::String => self.string(can_assign),
            ParseFn::Literal => self.literal(can_assign),
            ParseFn::And => self.and(can_assign),
            ParseFn::Or => self.or(can_assign),
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
    
    fn emit_byte(&mut self, byte: impl Into<u8>) {
        let line = self.previous_token.line;
        self.funpiler().chunk.write_byte(byte.into(), line);
    }
    
    fn emit_bytes(&mut self, byte1: impl Into<u8>, byte2: impl Into<u8>) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_op(OpCode::Constant);
        let constant_index = self.make_constant(value);
        self.emit_byte(constant_index);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant_index = self.funpiler().chunk.write_constant(value);
        if let Ok(index_u8) = u8::try_from(constant_index) {
            return index_u8;
        }
        else {
            self.error_at_current("Too many constants in one chunk. Max 256.");
            return 0;
        }
    }

    fn emit_back_jump(&mut self, landing: usize) {
        self.emit_byte(OpCode::JumpBack);
        let jump = self.funpiler().chunk.bytes.len() - landing + 2;
        if jump > u16::MAX.into() { self.error_at_current("Too much code to jump over."); }
        self.emit_byte(((jump >> 8) & 0xff) as u8);
        self.emit_byte((jump & 0xff) as u8);
    }

    fn emit_jump(&mut self, jump_op: OpCode) -> usize {
        self.emit_op(jump_op);
        self.emit_byte(0);
        self.emit_byte(0);
        return self.funpiler().chunk.bytes.len() - 2;
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.funpiler().chunk.bytes.len() - offset - 2;

        if jump > u16::MAX.into() { self.error_at_current("Too much code to jump over."); }
        self.funpiler().chunk.bytes[offset] = ((jump >> 8) & 0xff) as u8;
        self.funpiler().chunk.bytes[offset + 1] = (jump & 0xff) as u8;
    }
}

// Helpers
impl<'a> Compiler<'a> {

    /// Grabs the current funpiler from the top of the stack. </br>
    /// Will panic if there are no funpilers on the stack.
    fn funpiler(&mut self) -> &mut Funpiler {
        return self.funpiler_stack.last_mut().unwrap();
    }

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
        self.errors.push(CompilerError {
            line: token.line,
            start: token.start,
            len: token.length,
        });
        self.had_error = true;
    }

    fn error_at_previous(&mut self, message: &'static str) {
        self.error_at(self.previous_token, message);
    }
}

#[cfg(test)]
mod test {
    use crate::{chunk::Chunk, compiler::{Compiler, CompilerError}, opcode::OpCode, value::Value};

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
        assert_eq!(expected_chunk, output.expect("Failed to compile").script_function.chunk);
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
        assert_eq!(expected_chunk, output.expect("Failed to compile").script_function.chunk);
    }

    #[test]
    fn error_trailing_arithmetic_op() {
        let source = r#"1 +"#;
        let compiler = Compiler::new(&source);
        let expected_result = Err(vec![
            CompilerError { line: 1, start: 3, len: 0 }
        ]);

        let output = compiler.compile();
        assert_eq!(expected_result, output);
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
        assert_eq!(expected_chunk, output.expect("Failed to compile").script_function.chunk);
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
        assert_eq!(expected_chunk, output.expect("Failed to compile").script_function.chunk);
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
        assert_eq!(expected_chunk, output.expect("Failed to compile").script_function.chunk);
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
        assert_eq!(expected_chunk, output.expect("Failed to compile").script_function.chunk);
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
        assert_eq!(expected_chunk, output.script_function.chunk);
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
        assert_eq!(expected_chunk, output.script_function.chunk);
        assert_eq!(expected_global_count, output.globals_count);
    }

    #[test]
    fn error_redeclaration() {
        let source = r#"
var g = 1
var g = 2"#;
        let compiler = Compiler::new(&source);
        let expected_result = Err(vec![
            CompilerError { line: 3, start: 15, len: 1 }
        ]);

        let output = compiler.compile();
        assert_eq!(expected_result, output);
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
        assert_eq!(expected_chunk, output.script_function.chunk);
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
        assert_eq!(expected_chunk, output.script_function.chunk);
        assert_eq!(expected_global_count, output.globals_count);
    }
}