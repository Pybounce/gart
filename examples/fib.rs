use std::fs;

use gart::interpreter::Interpreter;


fn main() {
    let source: String = fs::read_to_string("examples/scripts/fib.gart").expect("Failed to read fib.gart file");
    let mut interpreter = Interpreter::new(source, Vec::new()).unwrap();
    match interpreter.run() {
        Ok(_) => (),
        Err(runtime_e) => println!("{:?}", runtime_e.message),
    }
}


