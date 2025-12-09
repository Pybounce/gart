use crate::{scanner, token::{Token, TokenType}};



pub struct Scanner<'a> {
    source: &'a str,
    /// The start of the token currently being scanned.(character index)
    start: usize,
    /// 1 past the most recently consumed character. (ie the next character (peek()))
    next: usize,
    line: usize,
    indent_stack: Vec<i32>,
    indent_target: i32,
    previous_token: Option<TokenType>
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            next: 0,
            line: 1,
            indent_stack: vec![0],
            indent_target: 0,
            previous_token: None
        }
    }
    pub fn scan_token(&mut self) -> Token {
        let token = self.next_token();
        self.previous_token = token.token_type.into();
        return token;
    }

}

impl<'a> Scanner<'a> {
        fn next_token(&mut self) -> Token {
        
        if let Some(token) = self.resolve_indent() {
            return token;
        }

        self.whitespace();

        if let Some(token) = self.newline() {
            return token;
        }

        self.start = self.next;

        let Some(c) = self.advance() else { return self.make_token(TokenType::Eof); };

        if self.is_alpha(c) { return self.identifier(); }
        if self.is_digit(c) { return self.number(); }
        match c {
            '(' => return self.make_token(TokenType::LeftParen),
            ')' => return self.make_token(TokenType::RightParen),
            ',' => return self.make_token(TokenType::Comma),
            '.' => return self.make_token(TokenType::Dot),
            '-' => return self.make_token(TokenType::Minus),
            '+' => return self.make_token(TokenType::Plus),
            '/' => return self.make_token(TokenType::Slash),
            '*' => return self.make_token(TokenType::Star),
            ':' => return self.make_token(TokenType::Colon),
            '!' => return if self.expect('=') { self.make_token(TokenType::BangEqual) } else { self.make_token(TokenType::Bang) },
            '=' => return if self.expect('=') { self.make_token(TokenType::EqualEqual) } else { self.make_token(TokenType::Equal) },
            '<' => return if self.expect('=') { self.make_token(TokenType::LessEqual) } else { self.make_token(TokenType::Less) },
            '>' => return if self.expect('=') { self.make_token(TokenType::GreaterEqual) } else { self.make_token(TokenType::Greater) },
            '"' => return self.string(),
            _ => {}
        }
        
        return self.make_err_token("Unexpect character.");
        
    }
    fn is_alpha(&self, c: char) -> bool {
        return (c >= 'a' && c <= 'z') ||
            (c >= 'A' && c <= 'Z') ||
            c == '_';
    }

    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn end_reached(&self) -> bool {
        return self.peek().is_none();
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        return Token::new(token_type, self.start as i32, (self.next - self.start) as i32, self.line as i32);
    }

    fn make_err_token(&self, message: &str) -> Token {
        println!("{}", message);
        return self.make_token(TokenType::Error);   // this is wrong but fine for now.
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(c) = self.peek() {
            self.next += c.len_utf8();
            return c.into();
        }
        return None;
    }

    fn peek(&self) -> Option<char> {
        return self.source[self.next..].chars().next();
    }

    fn peek_next(&self) -> Option<char> {
        if let Some(c) = self.peek() {
            return self.source[(self.next + c.len_utf8())..].chars().next();
        }
        return None;
    }

    fn expect(&mut self, expected: char) -> bool {
        if let Some(c) = self.peek() {
            if c != expected { return false;}
            self.advance();
            return true;
        }
        return false;
    }

    fn string(&mut self) -> Token {
        while let Some(c) = self.peek() {
            if c != '"' {
                if self.peek().unwrap() == '\n' { self.line += 1; }
                self.advance();
            }
            else {
                self.advance(); // consume the final ' " '
                return self.make_token(TokenType::String);
            }
        }
        return self.make_err_token("Unterminated string.");
    }

    fn number(&mut self) -> Token {
        while self.peek().is_some() && self.is_digit(self.peek().unwrap()) {
            self.advance();
        }
        if self.peek().is_some() && self.peek().unwrap() == '.' && self.peek_next().is_some() && self.is_digit(self.peek_next().unwrap()) {
            // consume '.'
            self.advance();

            while self.peek().is_some() && self.is_digit(self.peek().unwrap()) {
                self.advance();
            }
        }
        return self.make_token(TokenType::Number);
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_some() 
            && (self.is_alpha(self.peek().unwrap()) 
            || self.is_digit(self.peek().unwrap())) 
            { 
                self.advance(); 
            }
        return self.make_token(self.identifier_type());
    }

    fn identifier_type(&self) -> TokenType {
        let lexeme = self.lexeme(); // this is slow but fine for now.
        return match lexeme.as_str() {
            "and" => TokenType::And,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "null" => TokenType::Null,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier
        };
    }

    fn lexeme(&self) -> String {
        return self.source[self.start..self.next].to_string();
    }

    fn resolve_indent(&mut self) -> Option<Token> {
        if self.indent_stack.len() == 0 { 
            self.indent_stack.push(0);
        }
        let current_indent = *self.indent_stack.last().unwrap();

        if self.indent_target == current_indent { return None; }

        if self.indent_target > current_indent {
            self.indent_stack.push(self.indent_target);
            return self.make_token(TokenType::Indent).into();
        }

        if self.indent_target < current_indent {
            self.indent_stack.pop();
            if self.indent_stack.len() == 0 || self.indent_target > *self.indent_stack.last().unwrap() {
                // when dedenting, the target indent should always be a previous indent on the stack.
                return self.make_err_token("Inconsistent indent.").into();
            }
            else {
                return self.make_token(TokenType::Dedent).into();
            }
        }

        return self.make_err_token("Unreachable code reached.").into(); // unreachable
    }
    
    /// Checks for newline. </br>
    /// If found, processes spaces and tabs to create indent target and raises newline token </br>
    fn newline(&mut self) -> Option<Token> {
        if self.expect('\n') {
            let newline_token = self.make_token(TokenType::NewLine);
            self.line += 1;
            let mut col = 0;
            loop {
                let Some(c) = self.peek() else { 
                    self.indent_target = 0;
                    return newline_token.into(); 
                };
                match c {
                    ' ' => { col += 1; self.advance(); },
                    '\t' => { col += 4; self.advance(); },
                    '\n' => return self.newline(),
                    _ =>  {
                        let current = *self.indent_stack.last().unwrap_or(&0);
                        if let Some(prev_c) = self.previous_token {
                            if prev_c == TokenType::Colon {
                                println!("col   {}", col);
                                println!("cur   {}", current);
                                // target must be > current OR target must be 0
                                if col <= current {
                                    return self.make_err_token("Must indent the following code after ':'.").into();
                                }
                            }
                            else {
                                // target must be <= current OR the same
                                if col > current {
                                    return self.make_err_token("Cannot indent code after newline.").into();
                                }
                            }
                        }
                        self.indent_target = col;
                        return newline_token.into();
                    }
                };
            }
        }
        return None;
    }

    /// Skips all spaces and tabs
    fn whitespace(&mut self) {
        loop {
            let Some(c) = self.peek() else { return; };
            match c {
                ' ' => self.advance(),
                '\t' => self.advance(),
                _ => return
            };
        }
    }
}


#[cfg(test)]
mod test {
    use crate::{scanner::Scanner, token::{Token, TokenType}};


    #[test]
    fn single_statement() {
        let source = "var x = 1 + 1";
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Var, 0, 3, 1),
            Token::new(TokenType::Identifier, 4, 1, 1),
            Token::new(TokenType::Equal, 6, 1, 1),
            Token::new(TokenType::Number, 8, 1, 1),
            Token::new(TokenType::Plus, 10, 1, 1),
            Token::new(TokenType::Number, 12, 1, 1),
            Token::new(TokenType::Eof, 13, 0, 1),
        ];

        for expected_token in expected_tokens.iter() {
            assert_eq!(*expected_token, scanner.scan_token());
        }
    }

    #[test]
    fn error_random_indent() {
        let source = r#"
print "hello"
    print "world"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),

            Token::new(TokenType::Error, 0, 3, 1),

            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 0, 0),
            Token::new(TokenType::Eof, 13, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(expected_token.token_type, scanner.scan_token().token_type, "{}", i);  //temporary!
        }
    }

    #[test]
    fn error_empty_if() {
        let source = r#"
if x <= 1:
print "hi"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::If, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::LessEqual, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::Colon, 0, 3, 1),

            Token::new(TokenType::Error, 0, 3, 1),

            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 0, 0),
            Token::new(TokenType::Eof, 13, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(expected_token.token_type, scanner.scan_token().token_type, "{}", i);  //temporary!
        }
    }

    #[test]
    fn gap_in_indent() {
        let source = r#"
if x <= 1:
    print "hi"
    print "hello!"

    print "hola"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::If, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::LessEqual, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::Colon, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::Indent, 0, 3, 1),
            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Dedent, 0, 3, 1),

            Token::new(TokenType::Eof, 13, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(expected_token.token_type, scanner.scan_token().token_type, "{}", i);  //temporary!
        }
    }

    #[test]
    fn indented_newline() {
        // The empty line has an indentation. 
        // But since it ends in a newline with no other characters, it should be ignored.
        let source = r#"
if x <= 1:
    print "x greater than 1"
            
if x == 42:
    print "x is 42"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::If, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::LessEqual, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::Colon, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::Indent, 0, 3, 1),
            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Dedent, 0, 3, 1),

            Token::new(TokenType::If, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::EqualEqual, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::Colon, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),

            Token::new(TokenType::Indent, 0, 3, 1),
            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Dedent, 0, 3, 1),

            Token::new(TokenType::Eof, 13, 0, 1),
        ];

        for expected_token in expected_tokens.iter() {
            assert_eq!(expected_token.token_type, scanner.scan_token().token_type);  //temporary!
        }
    }

    #[test]
    fn indents() {
        let source = r#"
var x = 42
if x > 1:
    print "x greater than 1"
    if x == 42:
        print "x is 42"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Var, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::Equal, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::If, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::Greater, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::Colon, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Indent, 0, 3, 1),
            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::If, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::EqualEqual, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::Colon, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Indent, 0, 3, 1),
            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Dedent, 0, 3, 1),
            Token::new(TokenType::Dedent, 0, 3, 1),
            Token::new(TokenType::Eof, 0, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(expected_token.token_type, scanner.scan_token().token_type, "{}", i);  //temporary!
        }
    }

    #[test]
    fn whitespace() {
        let source = r#"
var    x   =     42
if  x    >   1:     
    print    "x greater than 1"
    if x    ==  42  :
        print "x is 42"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Var, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::Equal, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::If, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::Greater, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::Colon, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Indent, 0, 3, 1),
            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::If, 0, 3, 1),
            Token::new(TokenType::Identifier, 0, 3, 1),
            Token::new(TokenType::EqualEqual, 0, 3, 1),
            Token::new(TokenType::Number, 0, 3, 1),
            Token::new(TokenType::Colon, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Indent, 0, 3, 1),
            Token::new(TokenType::Print, 0, 3, 1),
            Token::new(TokenType::String, 0, 3, 1),
            Token::new(TokenType::NewLine, 0, 3, 1),
            Token::new(TokenType::Dedent, 0, 3, 1),
            Token::new(TokenType::Dedent, 0, 3, 1),
            Token::new(TokenType::Eof, 0, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(expected_token.token_type, scanner.scan_token().token_type, "{}", i);  //temporary!
        }
    }
}


