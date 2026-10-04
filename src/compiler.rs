use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;
use crate::vm::Bytecode;

pub struct Compiler {
    pub ast: Vec<statement::Statement>,
    pub bytecode: Bytecode
}

impl Compiler {

    pub fn new(ast: Vec<statement::Statement>) -> Self {
        Compiler {
            ast: ast.clone(),
            bytecode: Bytecode::new()
        }
    }

    pub fn compile(self: &mut Self) {

    }

    pub fn emit_byte(self: &mut Self, byte: u8) {
        self.bytecode.code.push(byte);
    }

    pub fn emit_return(self: &mut Self) {
        self.emit_byte(crate::OP_RETURN);
    }

    pub fn emit_bytes(self: &mut Self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    pub fn emit_constant(self: &mut Self, value: &Value) {
        let constant_idx = self.bytecode.write_constant(value.clone()) as u8;
        self.emit_bytes(crate::OP_CONSTANT, constant_idx);
    }

}
