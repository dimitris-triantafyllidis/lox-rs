use std::{env, fs, io, process::ExitCode};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

mod lexer;
mod parser;
mod interpreter;
mod context;
mod semantic_pass;
mod vm;

use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;
use crate::vm::*;

fn main() -> ExitCode {

    let mut vm = VirtualMachine::new();

    vm.values.push(Value::Number(10.0));

    vm.code = vec! [
       OP_RETURN,
       OP_CONSTANT,
       0
    ];

    vm.disassemble_chunk("test chunk".to_string());

    return ExitCode::from(0);
}

