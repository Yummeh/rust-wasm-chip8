mod audio;
mod cpu;
mod display;
mod keyboard_input;
mod timer;

use std::{cell::RefCell, path::Path, rc::Rc, u8, usize, vec};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    console::{self},
    Blob, Document, File as WasmFile, FileList, FileReader, HtmlCanvasElement, HtmlInputElement,
};

use self::display::Chip8WebGLDisplay;

const CHIP8_FILE_INPUT_DOC_ID: &str = "chip8-file-input";

pub enum Chip8Error {
    NoRomFound(String),
    DisplayFailed(String),
}

pub enum Chip8FileIOError {
    NoFileSelected,
}

// #[wasm_bindgen]
pub struct Chip8 {
    chip8_cpu: cpu::Chip8CPU,
    chip8_memory: Rc<RefCell<Chip8Memory>>,
    chip8_display: Rc<RefCell<Chip8WebGLDisplay>>,
    delay_timer: timer::Chip8DelayTimer,
    audio_timer: timer::Chip8AudioTimer,
    chip8_input: keyboard_input::Chip8Input,
    rom: Option<WasmFile>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mem = Rc::new(RefCell::new(Chip8Memory::new()));
        let disp = Rc::new(RefCell::new(Chip8WebGLDisplay::new("chip8_canvas")));
        // let ok = &*mem.borrow_mut();
        // let ok = disp.clone();

        Chip8 {
            rom: None,
            chip8_cpu: cpu::Chip8CPU::new(mem.clone(), disp.clone()),
            chip8_memory: mem.clone(),
            chip8_display: disp.clone(),
            delay_timer: timer::Chip8DelayTimer {},
            audio_timer: timer::Chip8AudioTimer {},
            chip8_input: keyboard_input::Chip8Input { num: 10 },
        }
    }

    pub fn set_rom(&mut self) -> Result<(), Chip8FileIOError> {
        let document: Document = web_sys::window().unwrap().document().unwrap();
        let file_input_element = document.get_element_by_id(&CHIP8_FILE_INPUT_DOC_ID).expect(
            format!(
                "Could not find file input by document id: {}",
                CHIP8_FILE_INPUT_DOC_ID
            )
            .as_str(),
        );

        let file_input_element: HtmlInputElement = file_input_element
            .dyn_into::<HtmlInputElement>()
            .map_err(|_| ())
            .unwrap();

        let file_list: FileList = file_input_element
            .files()
            .expect("Failed getting filelist.");

        let file: WasmFile = match file_list.item(0) {
            Some(file) => file,
            None => return Err(Chip8FileIOError::NoFileSelected),
        };

        self.rom = Some(file);

        Ok(())
    }

    // This might be a bad / disgusting way to read a binary file to memory, but it works. Using JS File.
    pub async fn load_rom(&mut self) {
        let file = match &self.rom {
            Some(rom) => rom,
            None => {
                panic!("Set rom please!");
            }
        };

        // Read file as binary to typed array.
        let array = match wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await {
            Ok(val) => val,
            Err(err) => {
                panic!("Failed smth");
            }
        };

        let new_array = js_sys::Uint8Array::new(&array);

        // Get memory stuff
        // let max = self.chip8_memory.data.len();
        // let buf = &mut self.chip8_memory.data[Chip8Memory::START_ADRESS as usize..max - 1];
        // let buf = &mut self.chip8_memory.borrow_mut().data[Chip8Memory::START_ADRESS as usize..max - 1];

        let memory = &mut self.chip8_memory.borrow_mut();
        let max = memory.data.len();
        let buf = &mut memory.data[Chip8Memory::START_ADRESS as usize..max - 1];

        // new_array does not have consistent length, indexing new_array with buf.len() as max range could be out of bounds.
        let iter_length;
        if (new_array.length()) <= buf.len() as u32 {
            iter_length = new_array.length();
        } else {
            iter_length = buf.len() as u32;
        }

        // Feels like bad implementation, O(n), but this might be the only option, doesn't really matter because files are only a couple kb big.
        // Read all binary data from array into chip8 memory
        for index in 0..iter_length {
            buf[index as usize] = new_array.get_index(index);
        }

        // To read binary data from regular Rust std::fs file
        // let max = self.memory.len();
        // let buf = &mut self.memory[Chip8Memory::START_ADRESS as usize..max - 1];
        // file.read(buf).unwrap();
    }

    pub fn test(&mut self) {
        let memory = self.chip8_memory.borrow_mut();
        let display = &mut self.chip8_display.borrow_mut();

        let width = 8u8;
        let height = 8u8;

        let x_pos = 0u8;
        let y_pos = 5u8;

        for i_y in 0..height {
            // let pixels = memory.data[(self.index + i_y as u16) as usize];

            for i_x in 0..width {
                // let mut pixel_state = (pixels >> i_x) & 0x1;

                // if pixel_state == 0 {
                //     pixel_state = 0x00;
                // } else {
                //     pixel_state = 0xFF;
                // }

                // display.set_pixel((i_x + x_pos) as u32, (i_y + y_pos) as u32, true);
                unsafe {
                    console::log_1(&JsValue::from_str(format!("x: {} y: {}", i_x, i_y).as_str()));
                }
                let flipped = display.xor_pixel(i_x, i_y, 0xFF);

                // if flipped {
                //     self.index_registers[0xF] = 1;
                // }
            }
        }

        display.draw();
    }

    pub fn update(&mut self, some_val: f32) {
        self.chip8_cpu.cycle();

        let disp = &mut self.chip8_display.borrow_mut();
        // disp.set_pixel(some_val as u32, 0, true);
        disp.draw();
    }
}

pub struct Chip8Memory {
    data: [u8; 4096],
}

impl Chip8Memory {
    pub const MEMORY_SIZE: usize = 4096;
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

    // pub fn load_

    pub fn new() -> Chip8Memory {
        let mut memory = Chip8Memory { data: [0; 4096] };

        for (index, byte) in Chip8Memory::FONT_SET.iter().enumerate() {
            memory.data[Chip8Memory::FONTSET_START_ADRESS as usize + index] = *byte;
        }

        memory
    }
}
