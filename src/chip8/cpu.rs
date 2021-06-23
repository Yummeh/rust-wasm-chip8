use std::u8;

pub struct Chip8CPU {
    index_registers: [u8; 16],
    pc: ProgramCounter,
    sp: StackPointer,
}

struct IndexRegister {}

struct ProgramCounter {}

struct StackPointer {}
