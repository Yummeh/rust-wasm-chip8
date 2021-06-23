mod audio;
mod cpu;
mod display;
mod keyboard_input;
mod timer;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Chip8 {
    cpu: cpu::Chip8CPU,
    memory: Chip8Memory,
    display: display::Chip8Display,
    delay_timer: timer::Chip8DelayTimer,
    audio_timer: timer::Chip8AudioTimer,
    input: keyboard_input::Chip8Input,
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            cpu: cpu::Chip8CPU {},
            memory: Chip8Memory {},
            display: display::Chip8Display {},
            delay_timer: timer::Chip8DelayTimer {},
            audio_timer: timer::Chip8AudioTimer {},
            input: keyboard_input::Chip8Input {},
        }
    }
}

struct Chip8Memory {}
