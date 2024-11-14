use crate::console_log;
use crate::ppu::GPU;
use std::rc::Rc;
use std::cell::RefCell;


#[derive(Clone)]
pub struct MemoryBus {
    rom: Vec<u8>,                 // Cartridge ROM
    vram: [u8; 0x2000],           // Video RAM
    eram: [u8; 0x2000],           // External RAM
    wram: [u8; 0x2000],           // Work RAM
    oam: [u8; 0xA0],              // Object Attribute Memory
    io_registers: [u8; 0x80],     // I/O Registers
    hram: [u8; 0x7F],             // High RAM (HRAM)
    pub interrupt_enable: u8,         // Interrupt Enable Register
    pub interrupt_flag: u8,           // Interrupt Flag Register (0xFF0F)
    pub gpu: Option<Rc<RefCell<GPU>>>,                  // Add a reference to your GPU
}

impl MemoryBus {
    pub fn new(rom: Vec<u8>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self  {
            rom,
            vram: [0; 0x2000],
            eram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io_registers: [0; 0x80],
            hram: [0; 0x7F],
            interrupt_enable: 0,
            interrupt_flag: 0,
            gpu: None
        }))
    }
    pub fn set_gpu(&mut self, gpu: Rc<RefCell<GPU>>) {
        self.gpu = Some(gpu);
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // ROM Bank 0 (0x0000 - 0x3FFF)
            0x0000..=0x3FFF => self.rom[address as usize],

            // ROM Bank 1 (Switchable) (0x4000 - 0x7FFF)
            0x4000..=0x7FFF => self.rom[address as usize], // Add bank switching logic if needed

            // Video RAM (0x8000 - 0x9FFF)
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],

            // External RAM (0xA000 - 0xBFFF)
            0xA000..=0xBFFF => self.eram[(address - 0xA000) as usize],

            // Work RAM (0xC000 - 0xCFFF)
            0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize],

            // Work RAM (0xD000 - 0xDFFF)
            0xD000..=0xDFFF => self.wram[(address - 0xC000) as usize],

            // Echo RAM (mirror of 0xC000 - 0xDDFF) (0xE000 - 0xFDFF)
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],

            // Object Attribute Memory (OAM) (0xFE00 - 0xFE9F)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            // Unused (0xFEA0 - 0xFEFF)
            0xFEA0..=0xFEFF => 0xFF,

            // I/O Registers (0xFF00 - 0xFF7F)
            0xFF00..=0xFF7F => {
                if address == 0xFF0F {
                    self.interrupt_flag
                } else if address == 0xFF40 {
                    self.gpu.as_ref().unwrap().borrow_mut().lcd_control
                }
                else {
                    self.io_registers[(address - 0xFF00) as usize]
                }
            }

            // High RAM (HRAM) (0xFF80 - 0xFFFE)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],

            // Interrupt Enable Register (0xFFFF)
            0xFFFF => self.interrupt_enable,

            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // ROM areas should be read-only
            0x0000..=0x7FFF => {
                // Optionally handle MBC here for bank switching
            }

            // Video RAM (0x8000 - 0x9FFF)
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,

            // External RAM (0xA000 - 0xBFFF)
            0xA000..=0xBFFF => self.eram[(address - 0xA000) as usize] = value,

            // Work RAM (0xC000 - 0xDFFF)
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,

            // Echo RAM (mirror of 0xC000 - 0xDDFF)
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,

            // Object Attribute Memory (OAM) (0xFE00 - 0xFE9F)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,

            // I/O Registers (0xFF00 - 0xFF7F)
            0xFF00..=0xFF7F => {
                if address == 0xFF0F {
                    self.interrupt_flag = value;
                } else if address == 0xFF40 {
                    // Update the LCD Control register
                    console_log!("Writing to LCD Control (0xFF40): {:#04X}", value);
                    self.gpu.as_mut().unwrap().borrow_mut().lcd_control = value;
                }
                else {
                    self.io_registers[(address - 0xFF00) as usize] = value;
                }
            }

            // High RAM (HRAM) (0xFF80 - 0xFFFE)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,

            // Interrupt Enable Register (0xFFFF)
            0xFFFF => self.interrupt_enable = value,

            _ => {} // Ignore writes to unmapped regions
        }
    }
}
