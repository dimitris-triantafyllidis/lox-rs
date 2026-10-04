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

}
