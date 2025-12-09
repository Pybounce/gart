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

        self.start = self.next;

        if let Some(token) = self.newline() {
            return token;
        }

        let Some(c) = self.advance() else { return self.make_token(TokenType::Eof); };

        if self.is_alpha(c) { return self.identifier(); }
        if self.is_digit(c) { return self.number(); }
        match c {
            '(' => return self.make_token(TokenType::LeftParen),
            ')' => return self.make_token(TokenType::RightParen),
            ',' => return self.make_token(TokenType::Comma),
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

    fn make_token(&mut self, token_type: TokenType) -> Token {
        let token = Token::new(token_type, self.start as i32, (self.next - self.start) as i32, self.line as i32);
        self.start = self.next;
        return token;
    }

    fn make_err_token(&mut self, message: &str) -> Token {
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
                    '\n' => {
                        let token = self.newline();
                        if let Some(t) = token {
                            if t.token_type == TokenType::Error {
                                return token;
                            }
                        }
                        return newline_token.into();
                    },
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
            Token::new(TokenType::NewLine, 0, 1, 1),

            Token::new(TokenType::Print, 1, 5, 2),
            Token::new(TokenType::String, 7, 7, 2),

            Token::new(TokenType::Error, 15, 4, 3),

            Token::new(TokenType::Print, 19, 5, 3),
            Token::new(TokenType::String, 25, 7, 3),
            Token::new(TokenType::NewLine, 32, 1, 3),
            Token::new(TokenType::Eof, 33, 0, 4),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
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
            Token::new(TokenType::NewLine, 0, 1, 1),

            Token::new(TokenType::If, 1, 2, 2),
            Token::new(TokenType::Identifier, 4, 1, 2),
            Token::new(TokenType::LessEqual, 6, 2, 2),
            Token::new(TokenType::Number, 9, 1, 2),
            Token::new(TokenType::Colon, 10, 1, 2),

            Token::new(TokenType::Error, 12, 0, 3),

            Token::new(TokenType::Print, 12, 5, 3),
            Token::new(TokenType::String, 18, 4, 3),
            Token::new(TokenType::NewLine, 22, 1, 3),
            Token::new(TokenType::Eof, 23, 0, 4),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
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
            Token::new(TokenType::NewLine, 0, 1, 1),

            Token::new(TokenType::If, 1, 2, 2),
            Token::new(TokenType::Identifier, 4, 1, 2),
            Token::new(TokenType::LessEqual, 6, 2, 2),
            Token::new(TokenType::Number, 9, 1, 2),
            Token::new(TokenType::Colon, 10, 1, 2),
            Token::new(TokenType::NewLine, 11, 1, 2),

            Token::new(TokenType::Indent, 12, 4, 3),
            Token::new(TokenType::Print, 16, 5, 3),
            Token::new(TokenType::String, 22, 4, 3),
            Token::new(TokenType::NewLine, 26, 1, 3),

            Token::new(TokenType::Print, 31, 5, 4),
            Token::new(TokenType::String, 37, 8, 4),
            Token::new(TokenType::NewLine, 45, 1, 4),

            Token::new(TokenType::Print, 51, 5, 6),
            Token::new(TokenType::String, 57, 6, 6),
            Token::new(TokenType::NewLine, 63, 1, 6),
            Token::new(TokenType::Dedent, 64, 0, 7),

            Token::new(TokenType::Eof, 64, 0, 7),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
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
    print "42"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 1, 1),

            Token::new(TokenType::If, 1, 2, 2),
            Token::new(TokenType::Identifier, 4, 1, 2),
            Token::new(TokenType::LessEqual, 6, 2, 2),
            Token::new(TokenType::Number, 9, 1, 2),
            Token::new(TokenType::Colon, 10, 1, 2),
            Token::new(TokenType::NewLine, 11, 1, 2),

            Token::new(TokenType::Indent, 12, 4, 3),
            Token::new(TokenType::Print, 16, 5, 3),
            Token::new(TokenType::String, 22, 18, 3),
            Token::new(TokenType::NewLine, 40, 1, 3),
            Token::new(TokenType::Dedent, 54, 0, 5),

            Token::new(TokenType::If, 54, 2, 5),
            Token::new(TokenType::Identifier, 57, 1, 5),
            Token::new(TokenType::EqualEqual, 59, 2, 5),
            Token::new(TokenType::Number, 62, 2, 5),
            Token::new(TokenType::Colon, 64, 1, 5),
            Token::new(TokenType::NewLine, 65, 1, 5),

            Token::new(TokenType::Indent, 66, 4, 6),
            Token::new(TokenType::Print, 70, 5, 6),
            Token::new(TokenType::String, 76, 4, 6),
            Token::new(TokenType::NewLine, 80, 1, 6),
            Token::new(TokenType::Dedent, 81, 0, 7),

            Token::new(TokenType::Eof, 81, 0, 7),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn nested_indents() {
        let source = r#"
var x = 42
if x > 1:
    print "x greater than 1"
    if x == 42:
        print "x is 42"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 1, 1),
            Token::new(TokenType::Var, 1, 3, 2),
            Token::new(TokenType::Identifier, 5, 1, 2),
            Token::new(TokenType::Equal, 7, 1, 2),
            Token::new(TokenType::Number, 9, 2, 2),
            Token::new(TokenType::NewLine, 11, 1, 2),
            Token::new(TokenType::If, 12, 2, 3),
            Token::new(TokenType::Identifier, 15, 1, 3),
            Token::new(TokenType::Greater, 17, 1, 3),
            Token::new(TokenType::Number, 19, 1, 3),
            Token::new(TokenType::Colon, 20, 1, 3),
            Token::new(TokenType::NewLine, 21, 1, 3),
            Token::new(TokenType::Indent, 22, 4, 4),
            Token::new(TokenType::Print, 26, 5, 4),
            Token::new(TokenType::String, 32, 18, 4),
            Token::new(TokenType::NewLine, 50, 1, 4),
            Token::new(TokenType::If, 55, 2, 5),
            Token::new(TokenType::Identifier, 58, 1, 5),
            Token::new(TokenType::EqualEqual, 60, 2, 5),
            Token::new(TokenType::Number, 63, 2, 5),
            Token::new(TokenType::Colon, 65, 1, 5),
            Token::new(TokenType::NewLine, 66, 1, 5),
            Token::new(TokenType::Indent, 67, 8, 6),
            Token::new(TokenType::Print, 75, 5, 6),
            Token::new(TokenType::String, 81, 9, 6),
            Token::new(TokenType::NewLine, 90, 1, 6),
            Token::new(TokenType::Dedent, 91, 0, 7),
            Token::new(TokenType::Dedent, 91, 0, 7),
            Token::new(TokenType::Eof, 91, 0, 7),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn whitespace() {
        let source = r#"
var    x   =     42
       
                 

print   "x is 42"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 1, 1),
            Token::new(TokenType::Var, 1, 3, 2),
            Token::new(TokenType::Identifier, 8, 1, 2),
            Token::new(TokenType::Equal, 12, 1, 2),
            Token::new(TokenType::Number, 18, 2, 2),
            Token::new(TokenType::NewLine, 20, 1, 2),

            Token::new(TokenType::Print, 48, 5, 6),
            Token::new(TokenType::String, 56, 9, 6),
            Token::new(TokenType::NewLine, 65, 1, 6),

            Token::new(TokenType::Eof, 66, 0, 7),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn error_unrecognised_token() {
        let source = r#"x = $"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Identifier, 0, 1, 1),
            Token::new(TokenType::Equal, 2, 1, 1),
            Token::new(TokenType::Error, 4, 1, 1),
            Token::new(TokenType::Eof, 5, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn error_unterminated_string() {
        let source = r#"x = "my_string"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Identifier, 0, 1, 1),
            Token::new(TokenType::Equal, 2, 1, 1),
            Token::new(TokenType::Error, 4, 10, 1),
            Token::new(TokenType::Eof, 14, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn multiline_string() {
        let source = r#"
x = "line 1
line 2"
"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::NewLine, 0, 1, 1),
            Token::new(TokenType::Identifier, 1, 1, 2),
            Token::new(TokenType::Equal, 3, 1, 2),
            Token::new(TokenType::String, 5, 15, 3),
            Token::new(TokenType::NewLine, 20, 1, 3),
            Token::new(TokenType::Eof, 21, 0, 4),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn keywords() {
        let source = r#"and else false for fn if null or print return true var while"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::And, 0, 3, 1),
            Token::new(TokenType::Else, 4, 4, 1),
            Token::new(TokenType::False, 9, 5, 1),
            Token::new(TokenType::For, 15, 3, 1),
            Token::new(TokenType::Fn, 19, 2, 1),
            Token::new(TokenType::If, 22, 2, 1),
            Token::new(TokenType::Null, 25, 4, 1),
            Token::new(TokenType::Or, 30, 2, 1),
            Token::new(TokenType::Print, 33, 5, 1),
            Token::new(TokenType::Return, 39, 6, 1),
            Token::new(TokenType::True, 46, 4, 1),
            Token::new(TokenType::Var, 51, 3, 1),
            Token::new(TokenType::While, 55, 5, 1),
            Token::new(TokenType::Eof, 60, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn identifiers_containing_keywords() {
        let source = r#"_and _else _false _for _fn if2 _null oor aprint _return true_ var_ _while_"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Identifier, 0, 4, 1),
            Token::new(TokenType::Identifier, 5, 5, 1),
            Token::new(TokenType::Identifier, 11, 6, 1),
            Token::new(TokenType::Identifier, 18, 4, 1),
            Token::new(TokenType::Identifier, 23, 3, 1),
            Token::new(TokenType::Identifier, 27, 3, 1),
            Token::new(TokenType::Identifier, 31, 5, 1),
            Token::new(TokenType::Identifier, 37, 3, 1),
            Token::new(TokenType::Identifier, 41, 6, 1),
            Token::new(TokenType::Identifier, 48, 7, 1),
            Token::new(TokenType::Identifier, 56, 5, 1),
            Token::new(TokenType::Identifier, 62, 4, 1),
            Token::new(TokenType::Identifier, 67, 7, 1),
            Token::new(TokenType::Eof, 74, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn operator_tokens() {
        let source = r#"+ - * / < > ! = <= >= != == and or"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Plus, 0, 1, 1),
            Token::new(TokenType::Minus, 2, 1, 1),
            Token::new(TokenType::Star, 4, 1, 1),
            Token::new(TokenType::Slash, 6, 1, 1),
            Token::new(TokenType::Less, 8, 1, 1),
            Token::new(TokenType::Greater, 10, 1, 1),
            Token::new(TokenType::Bang, 12, 1, 1),
            Token::new(TokenType::Equal, 14, 1, 1),
            Token::new(TokenType::LessEqual, 16, 2, 1),
            Token::new(TokenType::GreaterEqual, 19, 2, 1),
            Token::new(TokenType::BangEqual, 22, 2, 1),
            Token::new(TokenType::EqualEqual, 25, 2, 1),
            Token::new(TokenType::And, 28, 3, 1),
            Token::new(TokenType::Or, 32, 2, 1),
            Token::new(TokenType::Eof, 34, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn delimiter_tokens() {
        let source = r#": , ( )"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Colon, 0, 1, 1),
            Token::new(TokenType::Comma, 2, 1, 1),
            Token::new(TokenType::LeftParen, 4, 1, 1),
            Token::new(TokenType::RightParen, 6, 1, 1),
            Token::new(TokenType::Eof, 7, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn numbers() {
        let source = r#"2 24 2.394 0.1"#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Number, 0, 1, 1),
            Token::new(TokenType::Number, 2, 2, 1),
            Token::new(TokenType::Number, 5, 5, 1),
            Token::new(TokenType::Number, 11, 3, 1),
            Token::new(TokenType::Eof, 14, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn error_trailing_decimal() {
        let source = r#"var x = 2."#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Var, 0, 3, 1),
            Token::new(TokenType::Identifier, 4, 1, 1),
            Token::new(TokenType::Equal, 6, 1, 1),
            Token::new(TokenType::Number, 8, 1, 1),
            Token::new(TokenType::Error, 9, 1, 1),
            Token::new(TokenType::Eof, 10, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

    #[test]
    fn empty_source() {
        let source = r#""#;
        let mut scanner = Scanner::new(&source);

        let expected_tokens = vec![
            Token::new(TokenType::Eof, 0, 0, 1),
        ];

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(*expected_token, scanner.scan_token(), "Token Index: {}", i);
        }
    }

}


