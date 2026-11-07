use std::{env, fs, io, process::ExitCode};

use lox_rs::lexer as lexer;
use lox_rs::parser as parser;
use lox_rs::interpreter as interpreter;
use lox_rs::semantic_pass as semantic_pass;

fn main() -> ExitCode {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: lox [script]");
        return ExitCode::from(64);
    }
    else if args.len() == 2 {
        run_file(&args[1]);
    }

    return ExitCode::from(0);
}

fn run_file(file_path: &String) {

    match fs::read_to_string(file_path) {
        io::Result::Ok(s) => {
            let tokens = lexer::scan(&s);
            let mut syntax_tree = parser::parse(&tokens);
            let mut semantic_pass = semantic_pass::SemanticPass::new();
            semantic_pass.visit(&mut syntax_tree);
            let defs = semantic_pass.defs.clone();
            let mut interpreter = interpreter::Interpreter::new(semantic_pass.context);
            interpreter.execute(&syntax_tree, &defs);
        },
        io::Result::Err(e) => {
            eprintln!("io error: {}", e);
        }
    }

}
