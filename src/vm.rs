use crate::lexer::*;
use crate::parser::*;
use crate::interpreter::*;
use crate::semantic_pass::*;
use crate::compiler::*;

pub const OP_CONSTANT: u8 = 0;
pub const OP_RETURN:   u8 = 1;
pub const OP_NEGATE:   u8 = 2;
pub const OP_ADD:      u8 = 3;
pub const OP_SUBTRACT: u8 = 4;
pub const OP_MULTIPLY: u8 = 5;
pub const OP_DIVIDE:   u8 = 6;
pub const OP_NIL:      u8 = 7;
pub const OP_TRUE:     u8 = 8;
pub const OP_FALSE:    u8 = 9;
pub const OP_NOT:      u8 = 10;
pub const OP_EQUAL:    u8 = 11;
pub const OP_GREATER:  u8 = 12;
pub const OP_LESS:     u8 = 13;

#[derive(Debug, Clone, PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    pub code: Vec<u8>,
    pub values: Vec<Value>,
    pub lines: Vec<usize>
}

impl Bytecode {

    pub fn new() -> Self {
        Self {
            code: Vec::<u8>::new(),
            values: Vec::<Value>::new(),
            lines: Vec::<usize>::new()
        }
    }
}

impl Bytecode {

    pub fn disassemble_chunk(self: &Self, name: String) {

        println!("== {name} ==");

        let mut offset: usize = 0;

        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }

        println!();
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
            OP_NEGATE => {
                return self.simple_instruction("OP_NEGATE", offset);
            },
            OP_ADD => {
                return self.simple_instruction("OP_ADD", offset);
            },
            OP_SUBTRACT => {
                return self.simple_instruction("OP_SUBTRACT", offset);
            },
            OP_MULTIPLY => {
                return self.simple_instruction("OP_MULTIPLY", offset);
            },
            OP_DIVIDE => {
                return self.simple_instruction("OP_DIVIDE", offset);
            },
            OP_NIL => {
                return self.simple_instruction("OP_NIL", offset);
            },
            OP_TRUE => {
                return self.simple_instruction("OP_TRUE", offset);
            },
            OP_FALSE => {
                return self.simple_instruction("OP_FALSE", offset);
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
            Value::Boolean(b) => {
                println!("{b}");
            },
            Value::Number(n) => {
                println!("{n}");
            },
            Value::Nil => {
                println!("nil");
            },
            Value::String(s) => {
                println!("\"{s}\"");
            },
            Value::Function{..} => {
                println!("<fn>");
            },
            Value::Class{..} => {
                println!("<class>")
            },
            Value::Instance{..} => {
                println!("<instance>")
            }
        }
    }
}

pub struct VirtualMachine {
    pub program: Bytecode,
    pub stack: Vec<Value>,
    pub ip: usize,
    pub trace: bool
}

impl VirtualMachine {

    pub fn new() -> Self {
        Self {
            program: Bytecode::new(),
            stack: Vec::<Value>::new(),
            ip: 0,
            trace: true
        }
    }

    pub fn interpret(self: &mut Self) -> InterpretResult {
        return self.run();
    }

    pub fn read_code(self: &mut Self) -> u8 {
        let byte = self.program.code[self.ip];
        self.ip += 1;
        return byte;
    }

    pub fn read_constant(self: &mut Self) -> Value {
        let value = self.program.values[self.program.code[self.ip] as usize].clone();
        self.ip += 1;
        return value;
    }

    pub fn run(self: &mut Self) -> InterpretResult {

        loop {

            if self.trace {
                println!("        {:?}", self.stack);
                self.program.disassemble_instruction(self.ip);
            }

            let instruction = self.read_code();

            match instruction {
                OP_RETURN => {
                    let value = self.pop();
                    self.program.print_value(&value);
                    println!();
                    return InterpretResult::Ok
                },
                OP_CONSTANT => {
                    let value = self.read_constant();
                    self.push(value);
                },
                OP_NIL => {
                    self.push(Value::Nil);
                },
                OP_FALSE => {
                    self.push(Value::Boolean(false));
                },
                OP_TRUE => {
                    self.push(Value::Boolean(true));
                },
                OP_NEGATE => {
                    if let Value::Number(n) = self.pop() {
                        self.push(Value::Number(-n));
                    }
                    else {
                        panic!("Invalid operand type for OP_NEGATE");
                    }
                },
                OP_ADD => {
                    let vr = self.pop();
                    let vl = self.pop();
                    if let (Value::Number(l), Value::Number(r)) = (vl, vr) {
                        self.push(Value::Number(l + r));
                    }
                },
                OP_SUBTRACT => {
                    let vr = self.pop();
                    let vl = self.pop();
                    if let (Value::Number(l), Value::Number(r)) = (vl, vr) {
                        self.push(Value::Number(l - r));
                    }
                },
                OP_MULTIPLY => {
                    let vr = self.pop();
                    let vl = self.pop();
                    if let (Value::Number(l), Value::Number(r)) = (vl, vr) {
                        self.push(Value::Number(l * r));
                    }
                },
                OP_DIVIDE => {
                    let vr = self.pop();
                    let vl = self.pop();
                    if let (Value::Number(l), Value::Number(r)) = (vl, vr) {
                        self.push(Value::Number(l / r));
                    }
                },
                _ => {
                    panic!()
                }
            }
        }

        return InterpretResult::Ok;
    }

    pub fn push(self: &mut Self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(self: &mut Self) -> Value {
        return self.stack.pop().unwrap();
    }

}


