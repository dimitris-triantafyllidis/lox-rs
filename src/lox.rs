use std::{env, fs, io, process::ExitCode};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

mod lexer;
mod parser;
mod interpreter;
mod context;
mod semantic_pass;

use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;

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
            let tokens = lexer_scan(&s);
            let mut syntax_tree = parse(&tokens);
            let mut semantic_pass = SemanticPass::new();
            semantic_pass.visit(&mut syntax_tree);
            let mut interpreter = Interpreter::new();
            interpreter.execute(&syntax_tree);
        },
        io::Result::Err(e) => {
            eprintln!("io error: {}", e);
        }
    }

}
