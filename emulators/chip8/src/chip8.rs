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

pub struct Chip8 {
    memory: [u8; 4096],
    register: [u8; 16],
    stack: [u16; 16],
    sp: usize,
    pc: u16,
    display: [[bool; 64]; 32],
    keypad: [bool; 16]
}


impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            register: [0; 16],
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            display: [[false; 64]; 32],
            keypad: [false; 16]
        };

        chip8.initilize_memory();
        chip8

    }

    pub fn initilize_memory(&mut self) {
        self.memory[..FONTSET.len()].copy_from_slice(&FONTSET);
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
    
        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.OP_00E0(), // Clear display
            (0x0, 0x0, 0xE, 0xE) => self.OP_00EE(), // Return from subroutine
            (0x1, _, _, _) => self.OP_1nnn(nnn),   // Jump to address
            (0x2, _, _, _) => self.OP_2nnn(nnn),   // Call subroutine
            (0x3, _, _, _) => self.OP_3xkk(x, kk), // Skip if VX == kk
            (0x4, _, _, _) => self.OP_4xkk(x, kk), // Skip if VX != kk
            (0x5, _, _, 0x0) => self.OP_5xy0(x, y),// Skip if VX == VY
            (0x6, _, _, _) => self.OP_6xkk(x, kk), // Set VX = kk
            (0x7, _, _, _) => self.OP_7xkk(x, kk), // Add kk to VX
            (0x8, _, _, 0x0) => self.OP_8xy0(x, y),// Set VX = VY
            (0x8, _, _, 0x1) => self.OP_8xy1(x, y),// Set VX = VX OR VY
            (0x8, _, _, 0x2) => self.OP_8xy2(x, y),// Set VX = VX AND VY
            (0x8, _, _, 0x3) => self.OP_8xy3(x, y),// Set VX = VX XOR VY
            (0x8, _, _, 0x4) => self.OP_8xy4(x, y),// Add VY to VX, set VF = carry
            (0x8, _, _, 0x5) => self.OP_8xy5(x, y),// Subtract VY from VX, set VF = borrow
            (0x8, _, _, 0x6) => self.OP_8xy6(x),   // Shift VX right by 1
            (0x8, _, _, 0x7) => self.OP_8xy7(x, y),// Set VX = VY - VX
            (0x8, _, _, 0xE) => self.OP_8xyE(x),   // Shift VX left by 1
            (0x9, _, _, 0x0) => self.OP_9xy0(x, y),// Skip if VX != VY
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
        if self.sp > 0 {
            self.sp -= 1;
        }
        self.pc = self.stack[self.sp];
    }

    fn OP_2nnn(&mut self, address: u16) {
        self.stack[self.sp] = self.pc;
        self.sp +=1;
        self.pc = address;
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
        self.register[0xF] = if sum > 0xFF { 1 } else { 0 }; // Set carry flag
        self.register[vx as usize] = (sum & 0xFF) as u8; // Store the lower 8 bits
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



}