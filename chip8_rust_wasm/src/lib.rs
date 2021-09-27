use chip8::Chip8Error;
use js_sys::Date;
use std::cell::RefCell;
use std::f64;
use std::num::Wrapping;
use std::rc::Rc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventListener;
use web_sys::{console, Document, Element, HtmlCanvasElement, Window};

// use futures::executor::block_on;

// #![cfg(target_arch = "wasm32")]

// extern crate wasm_bindgen_test;

mod chip8;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Your code goes here!
    unsafe {
        console::log_1(&JsValue::from_str("Hello world!"));
    }

    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
pub fn test_comp() {
    // let val: u8 = 10;
    // let mut value: u8 = 0;
    // if true {
    //     let ok: u8 = 255;
    //     let result = val.wrapping_sub(value);
    //     value = result;
    // }

    // let random_value = rand::random::<u8>();

    let mut value: u16 = 975;

    for i in (0..3).rev() {
        let ok = value % 10;
        value /= 10;
        unsafe {
            console::log_1(&JsValue::from_str(format!("Value: {}", &ok).as_str()));
        }
    }
}

static mut BLOCK_START: bool = false;
static mut STOP_PROGRAM: bool = false;

#[wasm_bindgen]
pub fn stop_program() {
    unsafe {
        STOP_PROGRAM = true;
    }
}

#[wasm_bindgen]
pub async fn start() {
    unsafe {
        if BLOCK_START {
            console::log_1(&JsValue::from_str("Cannot start"));
            return;
        }
        BLOCK_START = true;
    }

    let mut chip8_emulator = chip8::Chip8::new();

    // match chip8_emulator.set_rom("Not right") {
    //     Ok(()) => {}
    //     Err(err) => unsafe {
    //         match err {
    //             chip8::Chip8FileIOError::NoFileSelected => {
    //                 console::log_1(&JsValue::from_str("Please select a file!"));
    //                 return;
    //             }
    //         }
    //     },
    // };

    // chip8_emulator.load_rom().await;

    // let result = chip8_emulator.update();
    // match result {
    //     Ok(()) => (),
    //     Err(error) => match error {
    //         Chip8Error::NoRomFound(error_message) => {
    //             panic!("{}", error_message);
    //         }
    //         Chip8Error::DisplayFailed(error_message) => {
    //             panic!("{}", error_message);
    //         }
    //     },
    // };

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let delay: i32 = 0;
    let mut last_cycle_time: f64 = Date::now();

    let mut i: f32 = 0.0;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let current_time: f64 = Date::now();
        let delta_time = current_time - last_cycle_time;

        if delta_time > delay as f64 {
            last_cycle_time = current_time;

            chip8_emulator.update(i);
            i += 1.0;
        }

        unsafe {
            if STOP_PROGRAM {
                let _ = f.borrow_mut().take();
                return;
            }
        }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    // chip8_emulator
    unsafe {
        STOP_PROGRAM = false;
    }
}

#[wasm_bindgen]
pub fn greet() {
    unsafe {
        console::log_1(&JsValue::from_str("Some greeting :D"));
    }
}

#[wasm_bindgen]
pub fn draw_to_canvas() {
    let document: Document = web_sys::window().unwrap().document().unwrap();
    let canvas: Element = document.get_element_by_id("chip8_canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    // return;
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.begin_path();

    // Draw the outer circle.
    context
        .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the mouth.
    context.move_to(110.0, 75.0);
    context.arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI).unwrap();

    // Draw the left eye.
    context.move_to(65.0, 65.0);
    context
        .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the right eye.
    context.move_to(95.0, 65.0);
    context
        .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    context.stroke();
}
