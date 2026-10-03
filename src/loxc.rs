use std::{env, fs, io, process::ExitCode};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

mod lexer;
mod parser;
mod interpreter;
mod context;
mod semantic_pass;
mod vm;
mod compiler;

use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;
use crate::vm::*;
use crate::compiler::*;

fn main() -> ExitCode {

    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: loxc [script]");
        return ExitCode::from(64);
    }
    else if args.len() == 2 {
        run_file(&args[1]);
    }
    else {
        run_repl();
    }

    return ExitCode::from(0);
}

fn run_file(file_path: &String) {

}

fn run_repl() {

    let mut rl = DefaultEditor::new().unwrap();

    loop {
        match rl.readline("lox > ") {

            Ok(line) => {
                let tokens = lexer_scan(&line);
                let parsed = parse(&tokens);
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                eprintln!("rustyline error: {}", e);
            }
        }
    }
}


