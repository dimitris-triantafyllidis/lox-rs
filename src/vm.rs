use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;

pub const OP_CONSTANT:  u8 = 0;
pub const OP_RETURN:    u8 = 1;



pub struct VirtualMachine {
    pub code: Vec<u8>,
    pub values: Vec<Value>,
    pub lines: Vec<usize>,
    pub ip: usize
}

impl VirtualMachine {

    pub fn new() -> Self {
        Self {
            code: Vec::<u8>::new(),
            values: Vec::<Value>::new(),
            lines: Vec::<usize>::new(),
            ip: 0
        }
    }

    pub fn disassemble_chunk(self: &Self, name: String) {

        println!("== {name} ==");

        let mut offset: usize = 0;

        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(self: &Self, offset: usize) -> usize {

        print!("{:04} ", offset);

        let insn = self.code[offset];

        match insn {
            OP_CONSTANT => {
                return self.constant_instruction("OP_CONSTANT", offset);
            },
            OP_RETURN => {
                return self.simple_instruction("OP_RETURN", offset);
            },
            _ => {
                println!("Unknown opcode: {insn}");
                return offset + 1;
            }
        }
    }

    pub fn simple_instruction(self: &Self, name: &str, offset: usize) -> usize {

        println!("{name}");
        return offset + 1;
    }

    pub fn constant_instruction(self: &Self, name: &str, offset: usize) -> usize {

        let constant_idx = self.code[offset + 1];
        print!("{:<-16} {:>4} ", name, constant_idx);
        self.print_value(&self.values[constant_idx as usize]);
        println!();

        return offset + 2;
    }

    pub fn print_value(self: &Self, value: &Value) {

        match value {
            Value::Number(n) => print!("'{n}'"),
            _ => panic!()
        }
    }

}


