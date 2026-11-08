use std::{env, fs, io, process::ExitCode};

use lox_rs::lexer;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: lox-scan [script]");
        return ExitCode::from(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    }

    ExitCode::from(0)
}

fn run_file(file_path: &str) {
    match fs::read_to_string(file_path) {
        io::Result::Ok(s) => {
            let tokens = lexer::scan(&s);
            lexer::print_token_stream(&tokens);
        }
        io::Result::Err(e) => {
            eprintln!("io error: {e}");
        }
    }
}
