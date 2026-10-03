use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;

pub const OP_CONSTANT:  u8 = 0;
pub const OP_RETURN:    u8 = 1;

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

pub struct VirtualMachine {
    pub code: Vec<u8>,
    pub values: Vec<Value>,
    pub lines: Vec<usize>,
    pub stack: Vec<Value>,
    pub ip: usize,
    pub trace: bool
}

impl VirtualMachine {

    pub fn new() -> Self {
        Self {
            code: Vec::<u8>::new(),
            values: Vec::<Value>::new(),
            lines: Vec::<usize>::new(),
            stack: Vec::<Value>::new(),
            ip: 0,
            trace: true
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

    pub fn write_code(self: &mut Self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_constant(self: &mut Self, value: Value) -> usize {
        self.values.push(value);
        return self.values.len() - 1;
    }

    pub fn interpret(self: &mut Self) -> InterpretResult {
        return self.run();
    }

    pub fn read_code(self: &mut Self) -> u8 {
        let byte = self.code[self.ip];
        self.ip += 1;
        return byte;
    }

    pub fn read_constant(self: &mut Self) -> Value {
        let value = self.values[self.code[self.ip] as usize].clone();
        self.ip += 1;
        return value;
    }

    pub fn run(self: &mut Self) -> InterpretResult {

        loop {

            if self.trace {
                println!("        {:?}", self.stack);
                self.disassemble_instruction(self.ip);
            }

            let instruction = self.read_code();

            match instruction {
                OP_RETURN => {
                    let value = self.pop();
                    self.print_value(&value);
                    return InterpretResult::Ok
                },
                OP_CONSTANT => {
                    let value = self.read_constant();
                    self.push(value);
                }
                _ => {
                    panic!()
                }
            }
        }

        return InterpretResult::Ok;
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

    pub fn push(self: &mut Self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(self: &mut Self) -> Value {
        return self.stack.pop().unwrap();
    }

}


