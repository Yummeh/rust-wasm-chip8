use std::{u16, u8, usize};

use super::{
    display::{Chip8WebGLDisplay},
    keyboard_input::Chip8Input,
    Chip8, Chip8Memory,
};

pub struct Chip8CPU {
    index_registers: [u8; 16],
    stack: [u16; 16],
    index: u16,
    sp: u8,
    pc: u16,
    opcode: u16,
    delay_timer: u8,
    sound_timer: u8,
    // table: [Fn; 10],
}

impl Chip8CPU {
    pub fn set_register(&mut self, index: &u8, value: &u8) {
        panic!("Function has not been implemented");
    }

    pub fn new() -> Chip8CPU {
        Chip8CPU {
            index_registers: [0; 16],
            stack: [0; 16],
            index: 0,
            sp: 0x00,
            pc: Chip8Memory::START_ADRESS as u16,
            opcode: 0x00,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn cycle(&mut self, memory: &mut Chip8Memory) {
        // Opcodes are stored in memory as 2 u8's so to get a u16 opcode use bitmask
        self.opcode = ((memory.data[self.pc as usize] as u16) << 8)
            | memory.data[self.pc as usize + 1] as u16;
        self.pc += 2;

        // Function pointer shit
        // TODO: Call function pointer table

        // Decrease delay timer
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        // Decrease sound timer
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    // Clear display
    fn op_00e0(&mut self, display: &mut Chip8WebGLDisplay) {
        display.clear();
    }

    // Return
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    // Jump to location NNN
    fn op_1nnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.pc = address;
    }

    // Call subroutine at NNN
    fn op_2nnn(&mut self) {
        // Add to stack
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;

        // Jump to location
        let address: u16 = self.opcode & 0x0FFF;
        self.pc = address;
    }

    // Skip next instruction if Vx == NN
    fn op_3xnn(&mut self) {
        let x = ((self.opcode >> 8) as u8) & 0x0F;
        let vx = self.index_registers[x as usize];
        let nn = self.opcode as u8;

        if vx == nn {
            self.pc += 2;
        }
    }

    // Skip next instruction if Vx != NN
    fn op_4xnn(&mut self) {
        let x = ((self.opcode >> 8) as u8) & 0x0F;
        let vx = self.index_registers[x as usize];
        let nn = self.opcode as u8;

        if vx != nn {
            self.pc += 2;
        }
    }

    // Skip next instruction if Vx == Vy
    fn op_5xy0(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx = self.index_registers[x as usize];
        let vy = self.index_registers[y as usize];

        if vx == vy {
            self.pc += 2;
        }
    }

    // Set Vx to NN
    fn op_6xnn(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let nn = self.opcode as u8;

        self.index_registers[x as usize] = nn;
    }

    // Add NN to Vx
    fn op_7xnn(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let nn = self.opcode as u8;

        self.index_registers[x as usize] += nn;
    }

    // Set Vx to value of Vy
    fn op_8xy0(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        self.index_registers[x as usize] = self.index_registers[y as usize];
    }

    // Set Vx to Vx Bitwise-OR Vy
    fn op_8xy1(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        self.index_registers[x as usize] |= self.index_registers[y as usize];
    }

    // Set Vx to Vx Bitwise-AND Vy
    fn op_8xy2(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        self.index_registers[x as usize] &= self.index_registers[y as usize];
    }

    // Set Vx to Vx Bitwise-XOR Vy
    fn op_8xy3(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        self.index_registers[x as usize] ^= self.index_registers[y as usize];
    }

    // Adds Vy to Vx. VF is set to 1 when there's a carry, and to 0 when there is not.
    fn op_8xy4(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        let result =
            self.index_registers[x as usize] as u16 + self.index_registers[y as usize] as u16;

        if result > 255 {
            self.index_registers[0xF_usize] = 1;
        } else {
            self.index_registers[0xF_usize] = 0;
        }

        self.index_registers[x as usize] = (result & 0xFF) as u8;
    }

    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_8xy5(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        if y > x {
            self.index_registers[0xF_usize] = 0;
        } else {
            self.index_registers[0xF_usize] = 1;
        }

        // Allow integer wrapping
        self.index_registers[x as usize] =
            self.index_registers[x as usize].wrapping_sub(self.index_registers[y as usize]);
    }

    // Stores the least significant bit of VX in VF and then shifts VX to the right by 1.
    fn op_8xy6(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        self.index_registers[0xF_usize] = self.index_registers[x as usize] & 0x1;
        self.index_registers[x as usize] >>= 1;
    }

    // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_8xy7(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        let vy = self.index_registers[y as usize];
        let vx = self.index_registers[x as usize];

        if vx > vy {
            self.index_registers[0xF_usize] = 0;
        } else {
            self.index_registers[0xF_usize] = 1;
        }

        self.index_registers[x as usize] = vy.wrapping_sub(vx);
    }

    // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
    fn op_8xye(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        self.index_registers[0xF_usize] = (self.index_registers[x as usize] & 0x80) >> 7;
        self.index_registers[x as usize] <<= 1;
    }

    // Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block);
    fn op_9xy0(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        if self.index_registers[x as usize] != self.index_registers[y as usize] {
            self.pc += 2;
        }
    }

    // Sets I to the address NNN.
    fn op_annn(&mut self) {
        let nnn: u16 = self.opcode & 0x0FFF;
        self.index = nnn;
    }

    // Jumps to the address NNN plus V0.
    fn op_bnnn(&mut self) {
        let nnn: u16 = self.opcode & 0x0FFF;
        self.pc = self.index_registers[0] as u16 + nnn;
    }

    // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    fn op_cxnn(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let nn = (self.opcode & 0xFF) as u8;

        self.index_registers[x as usize] = Chip8CPU::random_byte() & nn;
    }

    // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
    // Each row of 8 pixels is read as bit-coded starting from memory location I;
    // I value does not change after the execution of this instruction.
    // As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn,
    // and to 0 if that does not happen
    fn op_dxyn(&mut self, display: &mut Chip8WebGLDisplay, memory: &mut Chip8Memory) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        let x_pos = self.index_registers[x as usize] % Chip8WebGLDisplay::CHIP8_DISPLAY_WIDTH;
        let y_pos = self.index_registers[y as usize] & Chip8WebGLDisplay::CHIP8_DISPLAY_HEIGHT;

        let width = 8_u8;
        let height = (self.opcode & 0xF) as u8;

        self.index_registers[0xF] = 0;

        // Start at I (index) loop to I + y_index
        for i_y in 0..height {
            let pixels = memory.data[(self.index + i_y as u16) as usize];

            for i_x in 0..width {
                let mut pixel_state = (pixels >> i_x) & 0x1;

                if pixel_state == 0 {
                    pixel_state = 0x00;
                } else {
                    pixel_state = 0xFF;
                }

                let flipped = display.xor_pixel(i_x + x_pos, i_y + y_pos, pixel_state);

                if flipped {
                    self.index_registers[0xF] = 1;
                }
            }
        }
    }

    // Skips the next instruction if the key stored in VX is pressed.
    // (Usually the next instruction is a jump to skip a code block);
    fn op_ex9e(&mut self, input: &Chip8Input) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        // TODO: Implement input
        let input = 0;

        if self.index_registers[x as usize] == input {
            self.pc += 2;
        }
    }

    // Skips the next instruction if the key stored in VX is not pressed.
    // (Usually the next instruction is a jump to skip a code block);
    fn op_exa1(&mut self, input: &Chip8Input) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        // TODO: Implement input
        let input = 0;

        if self.index_registers[x as usize] != input {
            self.pc += 2;
        }
    }

    // Sets VX to the value of the delay timer.
    fn op_fx07(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        self.index_registers[x as usize] = self.delay_timer;
    }

    // A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event);
    fn op_fx0a(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        // TODO: Implement input
        let input = 0;

        if input == 0 {
            self.pc -= 2;
            return;
        }

        self.index_registers[x as usize] = input;
    }

    // Sets the delay timer to VX.
    fn op_fx15(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        self.delay_timer = self.index_registers[x as usize];
    }

    // Sets the sound timer to VX.
    fn op_fx18(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        self.sound_timer = self.index_registers[x as usize];
    }

    // Adds VX to I. VF is not affected.
    fn op_fx1e(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        self.index += self.index_registers[x as usize] as u16;
    }

    // Sets I to the location of the sprite for the character in VX.
    // Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    fn op_fx29(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        self.index = Chip8Memory::FONTSET_START_ADRESS as u16
            + (self.index_registers[x as usize] as u16 * 5);
    }

    // Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I,
    // the middle digit at I plus 1, and the least significant digit at I plus 2.
    // (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I,
    // the tens digit at location I+1, and the ones digit at location I+2.);
    fn op_fx33(&mut self, memory: &mut Chip8Memory) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let mut value = self.index_registers[x as usize];

        for i in (0..3).rev() {
            memory.data[(self.index + i) as usize] = value % 10;
            value /= 10;
        }
    }

    // Stores V0 to VX (including VX) in memory starting at address I.
    // The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    fn op_fx55(&mut self, memory: &mut Chip8Memory) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        for i in 0..x {
            memory.data[(self.index + i as u16) as usize] = self.index_registers[i as usize];
        }
    }

    // Fills V0 to VX (including VX) with values from memory starting at address I.
    // The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    fn op_fx65(&mut self, memory: &mut Chip8Memory) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        for i in 0..x {
            self.index_registers[i as usize] = memory.data[(self.index + i as u16) as usize];
        }
    }

    pub fn random_byte() -> u8 {
        rand::random::<u8>()
    }
}

// #[test]
// fn test_new_cpu() {
//     let cpu = Chip8CPU::new();

//     assert_eq!(cpu.index_registers[5], 0 as u8);
//     assert_eq!(cpu.index_registers[10], 0 as u8);

//     assert_eq!(cpu.pc, Chip8Memory::START_ADRESS as u16);
//     assert_eq!(cpu.sp, 0x0000);
// }
