use std::{u16, u8, usize};

use super::{Chip8, Chip8Memory, display::{self, Chip8Display}};

pub struct Chip8CPU {
    index_registers: [u8; 16],
    pc: u16,
    sp: u8,
}

impl Chip8CPU {
    pub fn set_register(&mut self, index: &u8, value: &u8) {
        panic!("Function has not been implemented");
    }

    pub fn new() -> Chip8CPU {
        Chip8CPU {
            index_registers: [0; 16],
            pc: Chip8Memory::START_ADRESS as u16,
            sp: 0x0000,
        }
    }

    pub fn start(
        &self,
        memory: &mut Chip8Memory,
        display: &mut Chip8Display,
        audio_timer: &mut super::timer::Chip8AudioTimer,
        delay_timer: &mut super::timer::Chip8DelayTimer,
        input: &mut super::keyboard_input::Chip8Input,
    ) {
        Chip8CPU::OP_00E0(display);
    }

    fn OP_00E0(display: &mut Chip8Display) {
        display.clear();
    }

    

    pub fn random_byte() -> u8 {
        rand::random::<u8>()
    }
}

#[test]
fn test_new_cpu() {
    let cpu = Chip8CPU::new();

    assert_eq!(cpu.index_registers[5], 0 as u8);
    assert_eq!(cpu.index_registers[10], 0 as u8);

    assert_eq!(cpu.pc, Chip8Memory::START_ADRESS as u16);
    assert_eq!(cpu.sp, 0x0000);
}
