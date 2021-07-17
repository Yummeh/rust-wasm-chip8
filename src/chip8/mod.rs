mod audio;
mod cpu;
mod display;
mod keyboard_input;
mod timer;

use std::{
    cell::RefCell,
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    path::Path,
    u8, usize, vec,
};

use wasm_bindgen::prelude::*;
use web_sys::console::assert;

use self::display::Chip8Display;
// ::{wasm_bindgen_test, wasm_bindgen_test_configure};

pub enum Chip8Error<'a> {
    NoRomFound(&'a str),
}

#[wasm_bindgen]
pub struct Chip8 {
    cpu: cpu::Chip8CPU,
    memory: Chip8Memory,
    display: display::Chip8Display,
    delay_timer: timer::Chip8DelayTimer,
    audio_timer: timer::Chip8AudioTimer,
    input: keyboard_input::Chip8Input,
    rom: Option<fs::File>,
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            rom: None,
            cpu: cpu::Chip8CPU::new(),
            memory: Chip8Memory::new(),
            display: Chip8Display::new(),
            delay_timer: timer::Chip8DelayTimer {},
            audio_timer: timer::Chip8AudioTimer {},
            input: keyboard_input::Chip8Input { num: 10 },
        }
    }

    pub fn set_rom<P>(&mut self, path: P) -> Result<(), std::io::Error>
    where
        P: AsRef<Path>,
    {
        self.rom = Some(File::open(path)?);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Chip8Error> {
        let rom = &mut self.rom;

        let mut file = if let Some(file) = rom {
            file
        } else {
            return Err(Chip8Error::NoRomFound(
                "No rom path specified, call set_rom!",
            ));
        };

        // File :D
        self.memory.load_rom(&mut file);

        self.cpu.start(
            &mut self.memory,
            &mut self.display,
            &mut self.audio_timer,
            &mut self.delay_timer,
            &mut self.input,
        );


        Ok(())
    }
}

pub struct Chip8Memory {
    memory: [u8; 4096],
}

impl Chip8Memory {
    pub const FONTSET_START_ADRESS: u32 = 0x50;
    pub const FONTSET_SIZE: u32 = 80;
    pub const START_ADRESS: u32 = 0x200;

    const FONT_SET: [u8; Chip8Memory::FONTSET_SIZE as usize] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    pub fn load_rom(&mut self, file: &mut File) {
        let max = self.memory.len();
        let buf = &mut self.memory[Chip8Memory::START_ADRESS as usize..max - 1];
        file.read(buf).unwrap();
    }

    pub fn new() -> Chip8Memory {
        let mut memory = Chip8Memory { memory: [0; 4096] };

        for (index, byte) in Chip8Memory::FONT_SET.iter().enumerate() {
            memory.memory[Chip8Memory::FONTSET_START_ADRESS as usize + index] = *byte;
        }

        memory
    }
}

#[test]
fn test_chip8_run() {
    let mut chip8 = Chip8::new();
    chip8
        .set_rom("src/test_stuff/TestBinary0-9")
        .expect("Could not find file");
    let result = chip8.run();

    match result {
        Ok(()) => (),
        Err(e) => match e {
            Chip8Error::NoRomFound(msg) => {
                panic!("{}", &msg);
            }
        },
    }
}

#[test]
fn test_load_rom() {
    let mut chip8_memory = Chip8Memory { memory: [0; 4096] };
    let mut file = File::open("src/test_stuff/TestBinary0-9").expect("Opening file failed");

    assert_eq!(chip8_memory.memory[0], 0);
    assert_eq!(chip8_memory.memory[400], 0);

    chip8_memory.load_rom(&mut file);

    assert_eq!(
        format!("{:02x}", 0x01),
        format!(
            "{:02x}",
            chip8_memory.memory[Chip8Memory::START_ADRESS as usize]
        )
    );
    assert_eq!(
        format!("{:02x}", 0x23),
        format!(
            "{:02x}",
            chip8_memory.memory[(Chip8Memory::START_ADRESS + 1) as usize]
        )
    );
    assert_eq!(
        format!("{:02x}", 0x89),
        format!(
            "{:02x}",
            chip8_memory.memory[(Chip8Memory::START_ADRESS + 4) as usize]
        )
    );
}

#[test]
fn test_new_chip8memory() {
    let chip8_memory = Chip8Memory::new();

    assert_eq!(
        chip8_memory.memory[Chip8Memory::FONTSET_START_ADRESS as usize],
        Chip8Memory::FONT_SET[0]
    );
    assert_eq!(
        chip8_memory.memory[Chip8Memory::FONTSET_START_ADRESS as usize + 1],
        Chip8Memory::FONT_SET[1]
    );
    assert_eq!(
        chip8_memory.memory
            [(Chip8Memory::FONTSET_START_ADRESS + Chip8Memory::FONTSET_SIZE - 1) as usize],
        Chip8Memory::FONT_SET[Chip8Memory::FONTSET_SIZE as usize - 1]
    );
}
