struct VirtualMachine {
    code: Vec<u8>,
    values: Vec<Value>,
    ip: usize
}
