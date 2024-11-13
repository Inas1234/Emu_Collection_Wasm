use getrandom::getrandom;
use crate::console_log;
const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

const FONTSET_START_ADDRESS: usize = 0x50;

pub struct Chip8 {
    memory: [u8; 4096],
    register: [u8; 16],
    stack: [u16; 16],
    sp: usize,
    index: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    pub display: [[bool; 64]; 32],
    pub keypad: [bool; 16],
    pub flattened_display: [bool; 64 * 32],
}


impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            register: [0; 16],
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            index: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [[false; 64]; 32],
            keypad: [false; 16],
            flattened_display: [false; 64 * 32]
        };

        chip8.initilize_memory();
        chip8

    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let start_address = 0x200;
        console_log!("Loading ROM of size {} bytes", rom.len());

        self.memory[start_address..start_address + rom.len()].copy_from_slice(rom);
        self.pc = start_address as u16;
        console_log!("Program counter set to: {:04X}", self.pc);

    }


    pub fn cycle(&mut self) {
        let opcode = self.opcode_fetch();
        console_log!("Fetched opcode: {:04X} at PC: {:04X}", opcode, self.pc);


        self.pc += 2;

        self.execute_opcode(opcode as u16);
    

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }


    pub fn initilize_memory(&mut self) {
        self.memory[FONTSET_START_ADDRESS..FONTSET_START_ADDRESS + FONTSET.len()]
        .copy_from_slice(&FONTSET);
    }

    pub fn opcode_fetch(&mut self) -> u16{
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte = self.memory[(self.pc + 1) as usize] as u16;

        (high_byte << 8) | low_byte
    }

    pub fn execute_opcode(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            opcode & 0x000F,
        );
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let x: u8 = nibbles.1 as u8;
        let y = nibbles.2 as u8;
        let n = nibbles.3 as u8;
    
        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.OP_00E0(),                // Clear display
            (0x0, 0x0, 0xE, 0xE) => self.OP_00EE(),                // Return from subroutine
            (0x1, _, _, _) => self.OP_1nnn(nnn),                  // Jump to address
            (0x2, _, _, _) => self.OP_2nnn(nnn),                  // Call subroutine
            (0x3, _, _, _) => self.OP_3xkk(x, kk),                // Skip if VX == kk
            (0x4, _, _, _) => self.OP_4xkk(x, kk),                // Skip if VX != kk
            (0x5, _, _, 0x0) => self.OP_5xy0(x, y),               // Skip if VX == VY
            (0x6, _, _, _) => self.OP_6xkk(x, kk),                // Set VX = kk
            (0x7, _, _, _) => self.OP_7xkk(x, kk),                // Add kk to VX
            (0x8, _, _, 0x0) => self.OP_8xy0(x, y),               // Set VX = VY
            (0x8, _, _, 0x1) => self.OP_8xy1(x, y),               // Set VX = VX OR VY
            (0x8, _, _, 0x2) => self.OP_8xy2(x, y),               // Set VX = VX AND VY
            (0x8, _, _, 0x3) => self.OP_8xy3(x, y),               // Set VX = VX XOR VY
            (0x8, _, _, 0x4) => self.OP_8xy4(x, y),               // Add VY to VX, set VF = carry
            (0x8, _, _, 0x5) => self.OP_8xy5(x, y),               // Subtract VY from VX, set VF = borrow
            (0x8, _, _, 0x6) => self.OP_8xy6(x),                  // Shift VX right by 1
            (0x8, _, _, 0x7) => self.OP_8xy7(x, y),               // Set VX = VY - VX
            (0x8, _, _, 0xE) => self.OP_8xyE(x),                  // Shift VX left by 1
            (0x9, _, _, 0x0) => self.OP_9xy0(x, y),               // Skip if VX != VY
            (0xA, _, _, _) => self.OP_Annn(nnn),                  // Set I = nnn
            (0xB, _, _, _) => self.OP_Bnnn(nnn),                  // Jump to V0 + nnn
            (0xC, _, _, _) => self.OP_Cxkk(kk, x),                // Set VX = random byte AND kk
            (0xD, _, _, _) => {self.OP_Dxyn(x, y, n);},              // Draw sprite
            (0xE, _, 0x9, 0xE) => self.OP_Ex9E(x),                // Skip if key in VX is pressed
            (0xE, _, 0xA, 0x1) => self.OP_ExA1(x),                // Skip if key in VX is not pressed
            (0xF, _, 0x0, 0x7) => self.OP_Fx07(x),                // Set VX = delay timer
            (0xF, _, 0x0, 0xA) => self.OP_Fx0A(x),                // Wait for key press
            (0xF, _, 0x1, 0x5) => self.OP_Fx15(x),                // Set delay timer = VX
            (0xF, _, 0x1, 0x8) => self.OP_Fx18(x),                // Set sound timer = VX
            (0xF, _, 0x1, 0xE) => self.OP_Fx1E(x),                // Set I = I + VX
            (0xF, _, 0x2, 0x9) => self.OP_Fx29(x),                // Set I = location of sprite for digit VX
            (0xF, _, 0x3, 0x3) => self.OP_Fx33(x),                // Store BCD of VX in memory at I
            (0xF, _, 0x5, 0x5) => self.OP_Fx55(x),                // Store registers V0 to VX in memory
            (0xF, _, 0x6, 0x5) => self.OP_Fx65(x),                // Load registers V0 to VX from memory
            _ => println!("Unknown opcode: {:04X}", opcode),
        }
    }
        
    fn OP_00E0(&mut self) {
        self.display = [[false; 64]; 32];
    }

    fn OP_1nnn(&mut self, address: u16) {
        self.pc = address;
    }

    fn OP_00EE(&mut self) {
        if self.sp == 0 {
            panic!("Stack underflow detected!");
        }
        
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    
        console_log!("Returning to address: {:04X}", self.pc);
    
        // self.pc += 2;
    }
        

    fn OP_2nnn(&mut self, address: u16) {
        if self.sp >= self.stack.len() {
            panic!("Stack overflow detected!");
        }
        self.stack[self.sp] = self.pc;
        self.sp += 1;
    
        self.pc = address;
        console_log!("Calling subroutine at address: {:04X}", address);
    }
    

    fn OP_3xkk(&mut self, vx: u8, address: u8) {
        if self.register[vx as usize] == address {
            self.pc += 2;
        }
    }

    fn OP_4xkk(&mut self, vx: u8, byte: u8) {
        if self.register[vx as usize] != byte {
            self.pc += 2;
        }
    }

    fn OP_5xy0(&mut self, vx: u8, vy: u8) {
        if self.register[vx as usize] != self.register[vy as usize] {
            self.pc += 2;
        }
    }

    fn OP_6xkk(&mut self, vx: u8, byte: u8) {
        self.register[vx as usize] = byte;
    }

    fn OP_7xkk(&mut self, vx: u8, byte: u8) {
        self.register[vx as usize] += byte;
    }   

    fn OP_8xy0(&mut self, vx: u8, vy: u8){
        self.register[vx as usize] = self.register[vy as usize];
    }

    fn OP_8xy1(&mut self, vx: u8, vy: u8){
        self.register[vx as usize] |= self.register[vy as usize];
    }

    fn OP_8xy2(&mut self, vx: u8, vy: u8){
        self.register[vx as usize] &= self.register[vy as usize];
    }

    fn OP_8xy3(&mut self, vx: u8, vy: u8){
        self.register[vx as usize] ^= self.register[vy as usize];
    }


    fn OP_8xy4(&mut self, vx: u8, vy: u8) {
        let sum: u16 = self.register[vx as usize] as u16 + self.register[vy as usize] as u16;
        self.register[0xF] = if sum > 0xFF { 1 } else { 0 }; 
        self.register[vx as usize] = (sum & 0xFF) as u8; 
    }
    
    fn OP_8xy5(&mut self, vx: u8, vy: u8){
        if self.register[vx as usize] > self.register[vy as usize] {
            self.register[0xF] = 1;
        }
        else {
            self.register[0xF] = 0;
        }
        self.register[vx as usize] -= self.register[vy as usize]

    }

    fn OP_8xy6(&mut self, vx: u8) {
        self.register[0xF] = (self.register[vx as usize] & 0x1);
        self.register[vx as usize] >>= 1;
    }


    fn OP_8xy7(&mut self, vx: u8, vy: u8){
        if self.register[vy as usize] > self.register[vx as usize] {
            self.register[0xF] = 1;
        }
        else {
            self.register[0xF] = 0;
        }
        self.register[vx as usize] = self.register[vy as usize] - self.register[vx as usize];

    }

    fn OP_8xyE(&mut self, vx: u8) {
        self.register[0xF] = (self.register[vx as usize] & 0x80) >> 7;
        self.register[vx as usize] <<= 1;
    }

    fn OP_9xy0(&mut self, vx: u8, vy: u8){
        if self.register[vx as usize] != self.register[vy as usize] {
            self.pc += 2;
        }
    }

    fn OP_Annn(&mut self, address: u16) {
        self.index = address;
    }

    fn OP_Bnnn(&mut self, address: u16) {
        console_log!("Jumping to address: {:04X} + V0 ({:02X})", address, self.register[0]);
        self.pc = (self.register[0] as u16 + address) as u16;
    }
      

    fn OP_Cxkk(&mut self, byte: u8, vx: u8) {
        let mut rand_byte = [0u8; 1];
        getrandom(&mut rand_byte).expect("Failed to generate a random byte");
        self.register[vx as usize] = rand_byte[0] & byte;
    }

    fn OP_Dxyn(&mut self, vx: u8, vy: u8, height: u8) {
        let x = self.register[vx as usize] as usize;
        let y = self.register[vy as usize] as usize;
    
        self.register[0xF] = 0;
    
        for byte_index in 0..height {
            let sprite_byte = self.memory[(self.index + byte_index as u16) as usize];
    
            for bit_index in 0..8 {
                let pixel = (sprite_byte >> (7 - bit_index)) & 1;
                let display_x = (x + bit_index) % 64;
                let display_y = (y + byte_index as usize) % 32;
                let index = display_y * 64 + display_x;
    
                // Check for collision
                if pixel == 1 {
                    if self.display[display_y][display_x] {
                        self.register[0xF] = 1;
                    }
                    self.display[display_y][display_x] ^= true;
                    self.flattened_display[index] = self.display[display_y][display_x];

                }
            }
        }
    }
        
    fn OP_Ex9E(&mut self, vx: u8) {
        let key = self.register[vx as usize];
        if self.keypad[key as usize] {
            self.pc += 2;
        }
    }

    fn OP_ExA1(&mut self, vx: u8) {
        let key = self.register[vx as usize];
        if !self.keypad[key as usize] {
            self.pc += 2;
        }
    }

    fn OP_Fx07(&mut self, vx: u8) {
        self.register[vx as usize] = self.delay_timer;
    }

    fn OP_Fx0A(&mut self, vx: u8) {
        for i in 0..self.keypad.len() {
            if self.keypad[i] {
                self.register[vx as usize] = i as u8;
                return; // Exit if a key is pressed
            }
        }
        // If no key is pressed, do not move the program counter
        self.pc -= 2;
    }
    
    fn OP_Fx15(&mut self, vx: u8) {
        self.delay_timer = self.register[vx as usize];
    }

    fn OP_Fx18(&mut self, vx: u8) {
        self.sound_timer = self.register[vx as usize];
    }

    fn OP_Fx1E(&mut self, vx: u8) {
        self.index += self.register[vx as usize] as u16;
    }

    fn OP_Fx29(&mut self, vx: u8) {
        let digit = self.register[vx as usize];
        self.index = (FONTSET_START_ADDRESS + (5 * digit as usize)) as u16;
    }
    
    fn OP_Fx33(&mut self, vx: u8) {
        let mut value = self.register[vx as usize];

        self.memory[self.index as usize + 2] = value % 10;
        value /= 10;

        self.memory[self.index as usize + 1] = value % 10;
        value /= 10;

        self.memory[self.index as usize] = value % 10;
    }

    fn OP_Fx55(&mut self, vx: u8) {
        for i in 0..=vx {
            self.memory[self.index as usize + i as usize] = self.register[i as usize];
        }
        self.index += vx as u16 + 1;
    }
    
    fn OP_Fx65(&mut self, vx: u8) {
        for i in 0..=vx {
            self.register[i as usize] = self.memory[self.index as usize + i as usize];
        }
        self.index += vx as u16 + 1;
    }
    

    

}