use std::fs;
use bytecode_vm::vm::VM;



fn main() {
    let source: String = fs::read_to_string("examples/scripts/test.dat").expect("Failed to read file");
    let mut vm = VM::new();
    vm.interpret(&source);
}


