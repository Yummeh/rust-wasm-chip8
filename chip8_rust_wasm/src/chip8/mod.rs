mod audio;
mod cpu;
mod display;
mod keyboard_input;
mod timer;

use std::{borrow::BorrowMut, cell::RefCell, path::Path, rc::Rc, u8, usize, vec};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    console::{self},
    Blob, Document, File as WasmFile, FileList, FileReader, HtmlCanvasElement, HtmlInputElement,
};

use self::display::Chip8WebGLDisplay;
// ::{wasm_bindgen_test, wasm_bindgen_test_configure};

const CHIP8_FILE_INPUT_DOC_ID: &str = "chip8-file-input";

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub enum Chip8Error {
    NoRomFound(String),
    DisplayFailed(String),
}

pub enum Chip8FileIOError {
    NoFileSelected,
}

#[wasm_bindgen]
pub struct Chip8 {
    chip8_cpu: cpu::Chip8CPU,
    chip8_memory: Chip8Memory,
    chip8_display: Chip8WebGLDisplay,
    delay_timer: timer::Chip8DelayTimer,
    audio_timer: timer::Chip8AudioTimer,
    chip8_input: keyboard_input::Chip8Input,
    rom: Option<WasmFile>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            rom: None,
            chip8_cpu: cpu::Chip8CPU::new(),
            chip8_memory: Chip8Memory::new(),
            chip8_display: Chip8WebGLDisplay::new("chip8_canvas"),
            delay_timer: timer::Chip8DelayTimer {},
            audio_timer: timer::Chip8AudioTimer {},
            chip8_input: keyboard_input::Chip8Input { num: 10 },
        }
    }

    pub fn set_rom<P>(&mut self, path: P) -> Result<(), Chip8FileIOError>
    where
        P: AsRef<Path>,
    {
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

        // Old stuff in comments
        // Creating a rust array, i.e. [u8, 4096],
        // create a new array with specified size -> read old array into new array -> convert js TypedArray to Rust array,
        // let array = js_sys::Uint8Array::new(&array);
        // let new_array = js_sys::Uint8Array::new_with_length(Chip8Memory::MEMORY_SIZE as u32);
        // new_array.set(&array, 0);
        // let mut rust_array: [u8; Chip8Memory::MEMORY_SIZE] = [0; Chip8Memory::MEMORY_SIZE];
        // new_array.copy_to(&mut rust_array);
        // let len = new_array.byte_length() as usize;

        let new_array = js_sys::Uint8Array::new(&array);

        // Get memory stuff
        let max = self.chip8_memory.data.len();
        let buf = &mut self.chip8_memory.data[Chip8Memory::START_ADRESS as usize..max - 1];

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

    pub fn update(&mut self, some_val: f32) {
        // self.chip8_display.set_pixel(some_val as i32, 0, true);
        // if some_val as i32 % 2 == 0 {
        //     self.chip8_display.set_checkerboard(true);
        // } else {
        //     self.chip8_display.set_checkerboard(false);

        // }
        self.chip8_display.set_pixel(some_val as u8, 0, true);
        self.chip8_display.draw();

        unsafe {
            // console::log_1(&JsValue::from_str(format!("Val: {}", some_val as i32).as_str()));
        }
        // let rom = &mut self.rom;

        // let mut file = if let Some(file) = rom {
        //     file
        // } else {
        //     return Err(Chip8Error::NoRomFound(
        //         "No rom path specified, call set_rom!".to_owned(),
        //     ));
        // };

        // // File :D

        // self.cpu.start(
        //     &mut self.memory,
        //     display,
        //     &mut self.audio_timer,
        //     &mut self.delay_timer,
        //     &mut self.input,
        // );

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

// #[test]
fn test_chip8_run() {
    let mut chip8 = Chip8::new();
    match chip8.set_rom("src/test_stuff/TestBinary0-9") {
        Ok(()) => {}
        Err(err) => unsafe {
            console::log_1(&JsValue::from_str("Please select a file!"));
            return;
        },
    }

    // let result = chip8.update();

}

// #[test]
// fn test_load_rom() {
//     let mut chip8_memory = Chip8Memory { memory: [0; 4096] };
//     let mut file = File::open("src/test_stuff/TestBinary0-9").expect("Opening file failed");

//     assert_eq!(chip8_memory.memory[0], 0);
//     assert_eq!(chip8_memory.memory[400], 0);

//     chip8_memory.load_rom(&mut file);

//     assert_eq!(
//         format!("{:02x}", 0x01),
//         format!(
//             "{:02x}",
//             chip8_memory.memory[Chip8Memory::START_ADRESS as usize]
//         )
//     );
//     assert_eq!(
//         format!("{:02x}", 0x23),
//         format!(
//             "{:02x}",
//             chip8_memory.memory[(Chip8Memory::START_ADRESS + 1) as usize]
//         )
//     );
//     assert_eq!(
//         format!("{:02x}", 0x89),
//         format!(
//             "{:02x}",
//             chip8_memory.memory[(Chip8Memory::START_ADRESS + 4) as usize]
//         )
//     );
// }

#[test]
fn test_new_chip8memory() {
    let chip8_memory = Chip8Memory::new();

    assert_eq!(
        chip8_memory.data[Chip8Memory::FONTSET_START_ADRESS as usize],
        Chip8Memory::FONT_SET[0]
    );
    assert_eq!(
        chip8_memory.data[Chip8Memory::FONTSET_START_ADRESS as usize + 1],
        Chip8Memory::FONT_SET[1]
    );
    assert_eq!(
        chip8_memory.data
            [(Chip8Memory::FONTSET_START_ADRESS + Chip8Memory::FONTSET_SIZE - 1) as usize],
        Chip8Memory::FONT_SET[Chip8Memory::FONTSET_SIZE as usize - 1]
    );
}
