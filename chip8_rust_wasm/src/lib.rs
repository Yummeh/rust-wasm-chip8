use js_sys::Date;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
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

fn set_timeout_with_callback(f: &Closure<dyn FnMut()>, delay: i32) -> i32 {
    let res = window()
        .set_interval_with_callback_and_timeout_and_arguments_0(f.as_ref().unchecked_ref(), delay)
        .expect("should register `requestAnimationFrame` OK");

    res
}

#[wasm_bindgen]
pub fn test_comp() {
    unsafe {
        // console::log_1(&JsValue::from_str(format!("{}", a).as_str()));
    }
}

static mut BLOCK_START: bool = false;
static mut STOP_PROGRAM: bool = false;
static mut TIMEOUT: i32 = 0;

#[wasm_bindgen]
pub fn stop_program() {
    unsafe {
        STOP_PROGRAM = true;
    }
}

#[wasm_bindgen]
pub async fn start() {
    unsafe {
        console::log_1(&JsValue::from_str("Hello"));
    }

    unsafe {
        if BLOCK_START {
            console::log_1(&JsValue::from_str("Cannot start"));
            return;
        }
        BLOCK_START = true;
    }

    let mut chip8_emulator = chip8::Chip8::new();

    // chip8_emulator.test();

    // return;

    match chip8_emulator.set_rom() {
        Ok(()) => {}
        Err(err) => unsafe {
            match err {
                chip8::Chip8FileIOError::NoFileSelected => {
                    console::log_1(&JsValue::from_str("Please select a file!"));
                    unsafe {
                        BLOCK_START = false;
                    }
                    return;
                }
            }
        },
    };

    chip8_emulator.load_rom().await;

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let delay1: i32 = 100;
    let delay2: i32 = 16;
    let mut last_cycle_time: f64 = Date::now();

    let mut i: f32 = 0.0;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // let current_time: f64 = Date::now();
        // let delta_time = current_time - last_cycle_time;

        // if delta_time > delay2 as f64 {
        //     last_cycle_time = current_time;

        chip8_emulator.update(i);
        //     i += 1.0;
        // }

        unsafe {
            if STOP_PROGRAM {
                STOP_PROGRAM = false;
                BLOCK_START = false;
                // window().clear_timeout_with_handle(TIMEOUT);
                let _ = f.borrow_mut().take();
                return;
            }
        }

        // set_timeout_with_callback(f.borrow().as_ref().unwrap(), delay1);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    // set_timeout_with_callback(g.borrow().as_ref().unwrap(), delay1);

    request_animation_frame(g.borrow().as_ref().unwrap());
}

#[wasm_bindgen]
pub fn greet() {
    unsafe {
        console::log_1(&JsValue::from_str("Some greeting :D"));
    }
}
