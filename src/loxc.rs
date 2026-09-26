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


    return ExitCode::from(0);
}

