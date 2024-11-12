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

    pub fn execute_opcode(opcode: u16) {

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


}