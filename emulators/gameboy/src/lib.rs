use wasm_bindgen::prelude::*;
pub use wasm_bindgen::memory; 
mod cpu;
mod memory;
mod ppu;
#[macro_use]
mod utils;

#[wasm_bindgen]
pub struct Emulator {
    cpu: cpu::CPU,
    gpu: ppu::GPU,
    memory: memory::MemoryBus,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(rom_data: Vec<u8>) -> Self {
        let memory = memory::MemoryBus::new(rom_data);

        let gpu = ppu::GPU::new(memory.clone());

        let cpu = cpu::CPU::new(memory.clone());

        Emulator { cpu, gpu, memory }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        self.memory = memory::MemoryBus::new(rom_data.clone());


        self.cpu = cpu::CPU::new(self.memory.clone());
    }


    pub fn step(&mut self) {
        self.cpu.step();
    }

    pub fn get_frame_buffer(&self) -> *const u8 {
        self.gpu.get_frame_buffer_ptr()
    }

    pub fn get_frame_buffer_length(&self) -> usize {
        self.gpu.get_frame_buffer_len()
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory.read_byte(address)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory.write_byte(address, value);
    }


}

