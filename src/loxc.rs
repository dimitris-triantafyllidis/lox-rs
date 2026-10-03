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

    vm.write_code(OP_RETURN, 1);
    vm.write_code(OP_CONSTANT, 1);

    let c_idx = vm.write_constant(Value::Number(5.0));

    vm.write_code(c_idx as u8, 1);

    vm.disassemble_chunk("test chunk".to_string());

    return ExitCode::from(0);
}

