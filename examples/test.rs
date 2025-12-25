use std::fs;

use bytecode_vm::interpreter::Interpreter;


fn main() {
    let source: String = fs::read_to_string("examples/scripts/test.dat").expect("Failed to read file");
    let mut interpreter = Interpreter::new(source, Vec::new()).unwrap();
    match interpreter.run() {
        Ok(_) => (),
        Err(runtime_e) => println!("{:?}", runtime_e.message),
    }
}


