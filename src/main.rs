
use std::io::{self, BufRead, Write};

use crate::vm::VM;

pub(crate) mod scanner;
pub(crate) mod token;
pub(crate) mod opcode;
pub(crate) mod chunk;
pub(crate) mod compiler;
pub(crate) mod value;
pub(crate) mod parse;
pub(crate) mod vm;

fn main() {
    println!("\n\n- - - - - - REPL MODE - - - - - -\n");
    
    let mut vm = VM::new();

    let mut source = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        match io::stdin().read_line(&mut source) {
            Ok(_) => {
                if !vm.interpret(&source) { 
                    println!("Failed to interpret input.");
                }
            }
            Err(e) => {
                println!("Error occured reading input. {:?}", e);
                return;
            }
        }
        source.clear();
    }

}
