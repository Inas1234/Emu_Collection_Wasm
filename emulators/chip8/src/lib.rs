use wasm_bindgen::prelude::*;
mod chip8;
#[macro_use]
mod utils;

use chip8::Chip8;
use std::cell::RefCell;
use std::rc::Rc;


thread_local! {
    static CHIP8: RefCell<Chip8> = RefCell::new(Chip8::new());
}

#[wasm_bindgen]
pub fn load_rom(rom: &[u8]) {
    CHIP8.with(|chip8| {
        chip8.borrow_mut().load_rom(rom);
    });
}

#[wasm_bindgen]
pub fn cycle() {
    CHIP8.with(|chip8| {
        chip8.borrow_mut().cycle();
    });
}

#[wasm_bindgen]
pub fn get_display_buffer() -> *const bool {
    CHIP8.with(|chip8| {
        let chip8 = chip8.borrow();
        chip8.flattened_display.as_ptr()
    })
}


#[wasm_bindgen]
pub fn key_down(key: u8) {
    CHIP8.with(|chip8| {
        chip8.borrow_mut().keypad[key as usize] = true;
    });
}

#[wasm_bindgen]
pub fn key_up(key: u8) {
    CHIP8.with(|chip8| {
        chip8.borrow_mut().keypad[key as usize] = false;
    });
}
