
use std::{env, fs, path::Path};

use gart::interpreter::Interpreter;


fn main() {
    let args: Vec<String> = env::args().collect();
    if !(args.len() == 2 || args.len() == 3) {
        println!("{}", args.len());
        for arg in args {
            println!("{}", arg);
        }
        println!("Unrecognised argument count.");
        println!("Use 'cargo run -r -- --help' for a list of commands");
        return;
    }
    match args[1].as_str() {
        "--path" => {
            if args.len() != 3 {
                println!("Unrecognised argument count.");
                println!("Use --help for a list of commands");
                return;
            }
            run_from_path(args[2].as_str());
        }
        "--help" => {
            println!("------ Usage ------");
            println!("Use 'cargo run -r -- --path [file_path]' to run any file.");
            println!("Use 'cargo run -r --example [example_name]' to run any built in example.");
            println!("Use 'cargo run -r -- --help' to display this message.");
        },
        _ => {
            println!("Unrecognised argument.");
            println!("Use --help for a list of commands");
        }
    };
}

fn run_from_path(path: &str) {
    let path = Path::new(path);
    let source: String = fs::read_to_string(path).expect("Failed to read file");
    if let Ok(mut interpreter) = Interpreter::new(source, Vec::new()) {
        match interpreter.run() {
            Ok(_) => (),
            Err(runtime_e) => println!("{:?}", runtime_e.message),
        }    
    }
    else {
        println!("Failed to compile.")
    }

}