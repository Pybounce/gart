use std::fs;

use gart::interpreter::Interpreter;


fn main() {
    let source: String = fs::read_to_string("examples/scripts/guessing_game.gart").expect("Failed to read guessing_game.gart file");
    let mut interpreter = Interpreter::new(source, Vec::new()).unwrap();
    match interpreter.run() {
        Ok(_) => (),
        Err(runtime_e) => println!("{:?}", runtime_e.message),
    }
}


