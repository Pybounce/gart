use std::env;

use crate::scanner::Scanner;

mod scanner;
mod token;

fn main() {
    let args: Vec<String> = env::args().collect();
    let source = "var x = 1 + 1";
    let mut scanner = Scanner::new(&source);
    scanner.scan_token();
}
