use wasm_bindgen::prelude::*;
pub use wasm_bindgen::memory; 
use std::rc::Rc;
use std::cell::RefCell;

mod cpu;
mod memory;
mod ppu;
#[macro_use]
mod utils;

#[wasm_bindgen]
pub struct Emulator {
    cpu: cpu::CPU,
    gpu: Rc<RefCell<ppu::GPU>>,
    memory: Rc<RefCell<memory::MemoryBus>>,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(rom_data: Vec<u8>) -> Self {
        // Step 1: Create MemoryBus without GPU reference
        let memory = memory::MemoryBus::new(rom_data);
        let memory_rc = Rc::clone(&memory);
        let gpu = ppu::GPU::new(Rc::clone(&memory_rc));

        // Step 3: Set the GPU reference in MemoryBus
        memory.borrow_mut().set_gpu(Rc::clone(&gpu));

        // Step 4: Create the CPU with references to both MemoryBus and GPU
        let cpu = cpu::CPU::new(Rc::clone(&memory_rc), Rc::clone(&gpu));

        Emulator { cpu: cpu, gpu: gpu, memory: memory }


    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        self.memory = memory::MemoryBus::new(rom_data.clone());
        self.gpu = ppu::GPU::new(self.memory.clone());

        self.cpu = cpu::CPU::new(self.memory.clone(), self.gpu.clone());

        self.gpu.borrow_mut().load_rom_to_vram(&rom_data);
        self.gpu.borrow_mut().setup_lcd_control();

        console_log!("ROM loaded, VRAM initialized, and LCD control set up");

    }


    pub fn step(&mut self) {
        self.cpu.step();
    }

    pub fn get_frame_buffer(&self) -> *const u8 {
        self.gpu.borrow_mut().get_frame_buffer_ptr()
    }

    pub fn get_frame_buffer_length(&self) -> usize {
        self.gpu.borrow_mut().get_frame_buffer_len()
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory.borrow_mut().read_byte(address)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory.borrow_mut().write_byte(address, value);
    }


}

