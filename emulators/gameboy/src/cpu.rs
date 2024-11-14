use crate::memory::MemoryBus;
use crate::ppu::GPU;
use crate::console_log;

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

#[derive(Clone, Copy)]
struct FlagRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool
}

impl std::convert::From<FlagRegister> for u8 {
    fn from(value: FlagRegister) -> Self {
        (if value.zero       { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION |
        (if value.subtract   { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION |
        (if value.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION |
        (if value.carry      { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagRegister {
    fn from(value: u8) -> Self {
        let zero = ((value >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((value >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((value >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((value >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagRegister {
            zero,
            subtract,
            half_carry,
            carry
        }
    }
    
}

struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    f: FlagRegister,

}

impl Registers {
    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | u8::from(self.f) as u16
    }

    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = FlagRegister::from((value & 0xFF) as u8);
    }

    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }
    
    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    fn reset(&mut self) {
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
        self.f = FlagRegister {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        };
    }


    fn read_register_8(&self, reg: u8) -> u8 {
        match reg {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => u8::from(self.f), 
            7 => self.a,
            _ => 0,
        }
    }

    fn write_register_8(&mut self, reg: u8, value: u8) {
        match reg {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => self.f = FlagRegister::from(value), 
            7 => self.a = value,
            _ => {}
        }
    }

}


#[derive(Debug, Clone, Copy)]
enum Instruction {
    ADD(ArithmeticTarget),        // Add register to A
    ADDHL,                        // Add 16-bit register to HL
    ADC(ArithmeticTarget),        // Add with carry
    SUB(ArithmeticTarget),        // Subtract from A
    SBC(ArithmeticTarget),        // Subtract with carry
    AND(ArithmeticTarget),        // Logical AND with A
    OR(ArithmeticTarget),         // Logical OR with A
    XOR(ArithmeticTarget),        // Logical XOR with A
    CP(ArithmeticTarget),         // Compare A with register
    INC(ArithmeticTarget),        // Increment register
    DEC(ArithmeticTarget),        // Decrement register
    CCF,                          // Complement Carry Flag
    SCF,                          // Set Carry Flag
    RRA,                          // Rotate A right through carry
    RLA,                          // Rotate A left through carry
    RRCA,                         // Rotate A right (no carry)
    RLCA,                         // Rotate A left (no carry)
    CPL,                          // Complement A
    BIT(u8, ArithmeticTarget),    // Test bit
    RESET(u8, ArithmeticTarget),  // Reset bit
    SET(u8, ArithmeticTarget),    // Set bit
    SRL(ArithmeticTarget),        // Shift right logical
    RR(ArithmeticTarget),         // Rotate right through carry
    RL(ArithmeticTarget),         // Rotate left through carry
    RRC(ArithmeticTarget),        // Rotate right (no carry)
    RLC(ArithmeticTarget),        // Rotate left (no carry)
    SRA(ArithmeticTarget),        // Shift right arithmetic
    SLA(ArithmeticTarget),        // Shift left arithmetic
    SWAP(ArithmeticTarget),       // Swap nibbles

    LD(ArithmeticTarget, ArithmeticTarget), // Load from one register to another
    LDFromSP(u16),                          // Load memory from stack pointer
    LDSPFromHL,                             // Load stack pointer from HL
    LDHLFromSP(i8),                         // Load HL from SP + signed offset
    POP(RegisterPair),                      // Pop value from stack into register pair
    PUSH(RegisterPair),
    PUSHAF,                    // Push value from register pair onto stack
    LDHLADecrement,
    INCHL,
    LDAFromHLIncrement,
    POPAF,
    INCBC,
        // New instructions for immediate values and memory access
    LDImmediate8(ArithmeticTarget, u8),
    LDImmediate16(RegisterPair, u16),
    LDIOOffsetFromA(u8),
    LDIOOffsetToA(u8),
    ADDImmediate(u8),
    ANDImmediate(u8),
    CPImmediate(u8),
    ADCImmediate(u8), // Add immediate value to A with carry
    SBCImmediate(u8), // Subtract immediate value from A with carry
    SUBImmediate(u8), // Subtract immediate value from A
    ORImmediate(u8),  // OR immediate value with A
    XORImmediate(u8), // XOR immediate value with A

    LDSPImmediate16(u16),
    LDAFromAddress(u16),
    LDAddressFromA(u16),
    JP(u16),
    NOP,
    HALT,
    DI,
    EI,

    JPConditional(Condition, u16),
    JR(i8),
    JRConditional(Condition, i8),
    CALL(u16),
    CALLConditional(Condition, u16),
    RET,
    RETConditional(Condition),
    RETI,
    RST(u8),
    STOP,
    JRZ(i8),
    JRNZ(i8),

}

#[derive(Clone, Copy, Debug)]
enum Condition {
    NZ,  // Not Zero
    Z,   // Zero
    NC,  // Not Carry
    C,   // Carry
}


#[derive(Clone, Copy, Debug)]
enum RegisterPair {
    BC,
    DE,
    HL,
}


#[derive(Clone, Copy, Debug)]
enum ArithmeticTarget {
    A, B, C, D, E, H, L,
}



pub struct CPU {
    registers: Registers,
    pc: u16,
    bus: MemoryBus,
    sp: u16,
    ime: bool,
    gpu: GPU
}

impl CPU {
    pub fn new(bus: MemoryBus) -> Self {
        let gpu = GPU::new(bus.clone());
        Self {
            registers: Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                h: 0, 
                l: 0,
                f: FlagRegister {
                    zero: false,
                    subtract: false,
                    half_carry: false,
                    carry: false
                },
            },
            pc: 0x0100,
            bus,
            sp: 0,
            ime: true,
            gpu
        }
    }

    fn handle_interrupts(&mut self) {
        let interrupt_enable = self.bus.read_byte(0xFFFF);
        let interrupt_flag = self.bus.read_byte(0xFF0F);

        let pending_interrupts = interrupt_enable & interrupt_flag;
        if pending_interrupts == 0 {
            return; 
        }

        self.ime = false;

        for (bit, vector) in [
            (0, 0x40), // VBlank
            (1, 0x48), // LCD STAT
            (2, 0x50), // Timer
            (3, 0x58), // Serial
            (4, 0x60), // Joypad
        ] {
            if (pending_interrupts & (1 << bit)) != 0 {
                self.service_interrupt(bit, vector);
                break; 
            }
        }
    }

    fn service_interrupt(&mut self, interrupt_bit: u8, vector: u16) {
        let interrupt_flag = self.bus.read_byte(0xFF0F);
        self.bus.write_byte(0xFF0F, interrupt_flag & !(1 << interrupt_bit));

        self.push_stack(self.pc);

        self.pc = vector;

        println!("Interrupt handled: bit {} -> vector {:04X}", interrupt_bit, vector);
    }

    fn push_stack(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
    }

    fn pop_stack(&mut self) -> u16 {
        let low = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let high = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        (high << 8) | low
    }

    pub fn step(&mut self) {
        // Check if interrupts are enabled, and handle them if so
        if self.ime {
            console_log!("Handling interrupts...");
            self.handle_interrupts();
        }
    
        // Read the opcode at the current program counter (PC)
        let opcode = self.bus.read_byte(self.pc);
        console_log!("Executing opcode: {:#04X} at PC: {:#04X}", opcode, self.pc);
    
        // Increment the program counter to point to the next instruction
        self.pc = self.pc.wrapping_add(1);
    
        // Decode the opcode into an instruction
        let instruction = self.decode_opcode(opcode);
        console_log!("Decoded instruction: {:?}", instruction);
    
        // Execute the decoded instruction and get the number of cycles taken
        let cycles = self.execute(instruction);
        console_log!("Executed instruction: {:?} took {} cycles", instruction, cycles);
    
        // Step the GPU with the number of cycles used by the instruction
        console_log!("Stepping GPU with {} cycles", cycles);
        self.gpu.step(cycles);
    
        console_log!("Step completed successfully\n");
    }
    
    fn decode_opcode(&mut self, opcode: u8) -> Instruction {
        match opcode {
            // NOP (No Operation)
            0x00 => Instruction::NOP,

            // ADD A, r
            0x80 => Instruction::ADD(ArithmeticTarget::B),
            0x81 => Instruction::ADD(ArithmeticTarget::C),
            0x82 => Instruction::ADD(ArithmeticTarget::D),
            0x83 => Instruction::ADD(ArithmeticTarget::E),
            0x84 => Instruction::ADD(ArithmeticTarget::H),
            0x85 => Instruction::ADD(ArithmeticTarget::L),
            0x87 => Instruction::ADD(ArithmeticTarget::A),

            // ADC A, r
            0x88 => Instruction::ADC(ArithmeticTarget::B),
            0x89 => Instruction::ADC(ArithmeticTarget::C),
            0x8A => Instruction::ADC(ArithmeticTarget::D),
            0x8B => Instruction::ADC(ArithmeticTarget::E),
            0x8C => Instruction::ADC(ArithmeticTarget::H),
            0x8D => Instruction::ADC(ArithmeticTarget::L),
            0x8F => Instruction::ADC(ArithmeticTarget::A),

            // SUB A, r
            0x90 => Instruction::SUB(ArithmeticTarget::B),
            0x91 => Instruction::SUB(ArithmeticTarget::C),
            0x92 => Instruction::SUB(ArithmeticTarget::D),
            0x93 => Instruction::SUB(ArithmeticTarget::E),
            0x94 => Instruction::SUB(ArithmeticTarget::H),
            0x95 => Instruction::SUB(ArithmeticTarget::L),
            0x97 => Instruction::SUB(ArithmeticTarget::A),

            // SBC A, r
            0x98 => Instruction::SBC(ArithmeticTarget::B),
            0x99 => Instruction::SBC(ArithmeticTarget::C),
            0x9A => Instruction::SBC(ArithmeticTarget::D),
            0x9B => Instruction::SBC(ArithmeticTarget::E),
            0x9C => Instruction::SBC(ArithmeticTarget::H),
            0x9D => Instruction::SBC(ArithmeticTarget::L),
            0x9F => Instruction::SBC(ArithmeticTarget::A),

            // AND A, r
            0xA0 => Instruction::AND(ArithmeticTarget::B),
            0xA1 => Instruction::AND(ArithmeticTarget::C),
            0xA2 => Instruction::AND(ArithmeticTarget::D),
            0xA3 => Instruction::AND(ArithmeticTarget::E),
            0xA4 => Instruction::AND(ArithmeticTarget::H),
            0xA5 => Instruction::AND(ArithmeticTarget::L),
            0xA7 => Instruction::AND(ArithmeticTarget::A),

            // OR A, r
            0xB0 => Instruction::OR(ArithmeticTarget::B),
            0xB1 => Instruction::OR(ArithmeticTarget::C),
            0xB2 => Instruction::OR(ArithmeticTarget::D),
            0xB3 => Instruction::OR(ArithmeticTarget::E),
            0xB4 => Instruction::OR(ArithmeticTarget::H),
            0xB5 => Instruction::OR(ArithmeticTarget::L),
            0xB7 => Instruction::OR(ArithmeticTarget::A),

            // XOR A, r
            0xA8 => Instruction::XOR(ArithmeticTarget::B),
            0xA9 => Instruction::XOR(ArithmeticTarget::C),
            0xAA => Instruction::XOR(ArithmeticTarget::D),
            0xAB => Instruction::XOR(ArithmeticTarget::E),
            0xAC => Instruction::XOR(ArithmeticTarget::H),
            0xAD => Instruction::XOR(ArithmeticTarget::L),
            0xAF => Instruction::XOR(ArithmeticTarget::A),

            // CP A, r
            0xB8 => Instruction::CP(ArithmeticTarget::B),
            0xB9 => Instruction::CP(ArithmeticTarget::C),
            0xBA => Instruction::CP(ArithmeticTarget::D),
            0xBB => Instruction::CP(ArithmeticTarget::E),
            0xBC => Instruction::CP(ArithmeticTarget::H),
            0xBD => Instruction::CP(ArithmeticTarget::L),
            0xBF => Instruction::CP(ArithmeticTarget::A),

            // INC r
            0x04 => Instruction::INC(ArithmeticTarget::B),
            0x0C => Instruction::INC(ArithmeticTarget::C),
            0x14 => Instruction::INC(ArithmeticTarget::D),
            0x1C => Instruction::INC(ArithmeticTarget::E),
            0x24 => Instruction::INC(ArithmeticTarget::H),
            0x2C => Instruction::INC(ArithmeticTarget::L),
            0x3C => Instruction::INC(ArithmeticTarget::A),

            // DEC r
            0x05 => Instruction::DEC(ArithmeticTarget::B),
            0x0D => Instruction::DEC(ArithmeticTarget::C),
            0x15 => Instruction::DEC(ArithmeticTarget::D),
            0x1D => Instruction::DEC(ArithmeticTarget::E),
            0x25 => Instruction::DEC(ArithmeticTarget::H),
            0x2D => Instruction::DEC(ArithmeticTarget::L),
            0x3D => Instruction::DEC(ArithmeticTarget::A),

            // CPL (Complement A)
            0x2F => Instruction::CPL,

            // CCF (Complement Carry Flag)
            0x3F => Instruction::CCF,

            // SCF (Set Carry Flag)
            0x37 => Instruction::SCF,

            // Rotate A instructions
            0x07 => Instruction::RLCA,
            0x0F => Instruction::RRCA,
            0x17 => Instruction::RLA,
            0x1F => Instruction::RRA,

            0x3E => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded LD A, {:#04X}", value);
                Instruction::LDImmediate8(ArithmeticTarget::A, value)
            }
            0xE0 => {
                let offset = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded LD (0xFF00 + {:#04X}), A", offset);
                Instruction::LDIOOffsetFromA(offset)
            }

            0x0E => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded LD C, {:#04X}", value);
                Instruction::LDImmediate8(ArithmeticTarget::C, value)
            }
    
    
            0x01 => {
                let value = self.fetch_word();
                Instruction::LDImmediate16(RegisterPair::BC, value)
            }
            
            0x06 => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded LD B, {:#04X}", value);
                Instruction::LDImmediate8(ArithmeticTarget::B, value)
            }
    
            0xC6 => {
                let value = self.fetch_byte();
                Instruction::ADDImmediate(value)
            }
    
            0xFA => {
                let address = self.fetch_word();
                Instruction::LDAFromAddress(address)
            }
    
            0xEA => {
                let low = self.bus.read_byte(self.pc);
                let high = self.bus.read_byte(self.pc + 1);
                let address = ((high as u16) << 8) | (low as u16);
                self.pc = self.pc.wrapping_add(2);
                console_log!("Decoded LD (nn), A with address {:#04X}", address);
                Instruction::LDAddressFromA(address)
            }
            0x7C => Instruction::LD(ArithmeticTarget::A, ArithmeticTarget::H),

            0xF3 => {
                console_log!("Decoded DI (Disable Interrupts)");
                Instruction::DI
            }
    
            0xC3 => {
                let low = self.bus.read_byte(self.pc);
                let high = self.bus.read_byte(self.pc + 1);
                let address = ((high as u16) << 8) | (low as u16);
                console_log!("Decoded JP to address: {:#04X}", address);
                self.pc = self.pc.wrapping_add(2); 
                Instruction::JP(address)
            }

            0x21 => {
                let low = self.bus.read_byte(self.pc);
                let high = self.bus.read_byte(self.pc + 1);
                let value = ((high as u16) << 8) | (low as u16);
                console_log!("Decoded LD HL, {:#04X}", value);
                self.pc = self.pc.wrapping_add(2);
                Instruction::LDImmediate16(RegisterPair::HL, value)
            }
    
            
            0xF0 => {
                let offset = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded LD A, (0xFF00 + {:#04X})", offset);
                Instruction::LDIOOffsetToA(offset)
            }

            0xE6 => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded AND {:#04X}", value);
                Instruction::ANDImmediate(value)
            }

            0xFE => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded CP {:#04X}", value);
                Instruction::CPImmediate(value)
            }
    
    
            0x06 => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                Instruction::LDImmediate8(ArithmeticTarget::B, value)
            }
            0x0E => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                Instruction::LDImmediate8(ArithmeticTarget::C, value)
            }
            0x21 => {
                let low = self.bus.read_byte(self.pc);
                let high = self.bus.read_byte(self.pc + 1);
                let value = ((high as u16) << 8) | low as u16;
                self.pc = self.pc.wrapping_add(2);
                Instruction::LDImmediate16(RegisterPair::HL, value)
            }

            0xC3 => {
                let low = self.bus.read_byte(self.pc);
                let high = self.bus.read_byte(self.pc + 1);
                let address = ((high as u16) << 8) | low as u16;
                self.pc = self.pc.wrapping_add(2);
                Instruction::JP(address)
            }
            0xC9 => Instruction::RET,
            0xCD => {
                let low = self.bus.read_byte(self.pc);
                let high = self.bus.read_byte(self.pc + 1);
                let address = ((high as u16) << 8) | low as u16;
                self.pc = self.pc.wrapping_add(2);
                Instruction::CALL(address)
            }
            0xE0 => {
                let offset = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                Instruction::LDIOOffsetFromA(offset)
            }
            0xE6 => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                Instruction::ANDImmediate(value)
            }
            0xF0 => {
                let offset = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                Instruction::LDIOOffsetToA(offset)
            }
            0xF3 => Instruction::DI,
            0xFB => Instruction::EI,
            0xFE => {
                let value = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                Instruction::CPImmediate(value)
            }
            0x32 => Instruction::LDHLADecrement,

            // Halt (stop CPU until an interrupt occurs)
            0x76 => Instruction::HALT,
            0x31 => {
                // Read the next two bytes to form a 16-bit value
                let low = self.bus.read_byte(self.pc);
                let high = self.bus.read_byte(self.pc + 1);
                let value = ((high as u16) << 8) | (low as u16);
                self.pc = self.pc.wrapping_add(2); // Advance the program counter by 2 bytes
                console_log!("Decoded LD SP, {:#04X}", value);
                Instruction::LDSPImmediate16(value)
            }
            0x7D => Instruction::LD(ArithmeticTarget::A, ArithmeticTarget::L),
            0x18 => {
                // Read the signed 8-bit offset
                let offset = self.bus.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded JR {:+}", offset);
                Instruction::JR(offset)
            }
            0xE5 => Instruction::PUSH(RegisterPair::HL),
            0xE1 => Instruction::POP(RegisterPair::HL),
            0xF5 => Instruction::PUSHAF, // Handle AF separately
            0x23 => Instruction::INCHL,
            0x2A => Instruction::LDAFromHLIncrement,
            0xF1 => Instruction::POPAF,
            0xC5 => Instruction::PUSH(RegisterPair::BC),
            0x03 => Instruction::INCBC,
            0x78 => Instruction::LD(ArithmeticTarget::A, ArithmeticTarget::B),
            0x28 => {
                // Read the signed 8-bit offset
                let offset = self.bus.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded JR Z, {:+}", offset);
                Instruction::JRZ(offset)
            }
            0x20 => {
                // Read the signed 8-bit offset
                let offset = self.bus.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(1);
                console_log!("Decoded JR NZ, {:+}", offset);
                Instruction::JRNZ(offset)
            }
    
    
            // Extended opcodes (0xCB prefix)
            0xCB => self.decode_extended_opcode(),

            _ => panic!("Unknown opcode: 0x{:02X}", opcode),
        }
    }

    fn fetch_byte(&mut self) -> u8 {
        let value = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn fetch_word(&mut self) -> u16 {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;
        (high << 8) | low
    }


    fn decode_extended_opcode(&mut self) -> Instruction {
        let opcode = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        match opcode {
            0x11 => Instruction::RL(ArithmeticTarget::C),
            0x12 => Instruction::RL(ArithmeticTarget::D),
            0x13 => Instruction::RL(ArithmeticTarget::E),
            0x14 => Instruction::RL(ArithmeticTarget::H),
            0x15 => Instruction::RL(ArithmeticTarget::L),
            _ => panic!("Unknown extended opcode: 0xCB{:02X}", opcode),
        }
    }

    fn execute(&mut self, instruction: Instruction) -> u32 {
        match instruction {
            Instruction::ADD(target) => { self.add(target); 4 },
            Instruction::ADDHL => { self.add_hl(); 8 },
            Instruction::ADC(target) => { self.adc(target); 4 },
            Instruction::SUB(target) => { self.sub(target); 4 },
            Instruction::SBC(target) => { self.sbc(target); 4 },
            Instruction::AND(target) => { self.and(target); 4 },
            Instruction::OR(target) => { self.or(target); 4 },
            Instruction::XOR(target) => { self.xor(target); 4 },
            Instruction::CP(target) => { self.cp(target); 4 },
            Instruction::INC(target) => { self.inc(target); 4 },
            Instruction::DEC(target) => { self.dec(target); 4 },
            Instruction::CCF => { self.ccf(); 4 },
            Instruction::SCF => { self.scf(); 4 },
            Instruction::RRA => { self.rra(); 4 },
            Instruction::RLA => { self.rla(); 4 },
            Instruction::RRCA => { self.rrca(); 4 },
            Instruction::RLCA => { self.rlca(); 4 },
            Instruction::CPL => { self.cpl(); 4 },
            Instruction::BIT(bit, target) => { self.bit(bit, target); 8 },
            Instruction::RESET(bit, target) => { self.reset_bit(bit, target); 8 },
            Instruction::SET(bit, target) => { self.set_bit(bit, target); 8 },
            Instruction::SRL(target) => { self.srl(target); 8 },
            Instruction::RR(target) => { self.rr(target); 8 },
            Instruction::RL(target) => { self.rl(target); 8 },
            Instruction::RRC(target) => { self.rrc(target); 8 },
            Instruction::RLC(target) => { self.rlc(target); 8 },
            Instruction::SRA(target) => { self.sra(target); 8 },
            Instruction::SLA(target) => { self.sla(target); 8 },
            Instruction::SWAP(target) => { self.swap(target); 8 },
            Instruction::LDImmediate8(target, value) => { self.set_register_value(target, value); 8 },
            Instruction::LDImmediate16(pair, value) => { self.set_register_pair(pair, value); 12 },
            Instruction::ADDImmediate(value) => {
                let (result, carry) = self.registers.a.overflowing_add(value);
                self.update_flags(result, false, carry, (self.registers.a & 0xF) + (value & 0xF) > 0xF);
                self.registers.a = result;
                8
            }
            Instruction::LDAFromAddress(address) => {
                let value = self.bus.read_byte(address);
                self.registers.a = value;
                16
            }
            Instruction::LDAddressFromA(address) => {
                self.bus.write_byte(address, self.registers.a);
                16
            }

            Instruction::LDIOOffsetFromA(offset) => {
                let address = 0xFF00 + (offset as u16);
                console_log!("Executing LD (0xFF00 + {:#04X}), A", offset);
                self.bus.write_byte(address, self.registers.a);
                12 // LD (0xFF00 + n), A takes 12 cycles
            }    
            Instruction::LDIOOffsetToA(offset) => {
                let address = 0xFF00 + (offset as u16);
                console_log!("Executing LD A, (0xFF00 + {:#04X})", offset);
                self.registers.a = self.bus.read_byte(address);
                console_log!("Loaded value {:#04X} into A from address {:#04X}", self.registers.a, address);
                12 // LD A, (0xFF00 + n) takes 12 cycles
            }    
    
            Instruction::HALT => {
                println!("CPU HALT");
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            Instruction::LD(ArithmeticTarget::A, ArithmeticTarget::L) => {
                console_log!("Executing LD A, L");
                self.registers.a = self.registers.l;
                4 // LD A, L takes 4 cycles
            }
            Instruction::LD(ArithmeticTarget::A, ArithmeticTarget::B) => {
                console_log!("Executing LD A, B");
    
                // Copy the value from register B to register A
                self.registers.a = self.registers.b;
                console_log!("Loaded value {:#04X} from B into A", self.registers.a);
    
                4 // LD A, B takes 4 cycles
            }
    
    
            Instruction::ANDImmediate(value) => {
                console_log!("Executing AND {:#04X} with A = {:#04X}", value, self.registers.a);
                
                // Perform the AND operation
                self.registers.a &= value;
                
                // Update flags
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
                self.registers.f.carry = false;
    
                console_log!("Result of AND: {:#04X}, Flags - Z:{} N:{} H:{} C:{}",
                    self.registers.a,
                    self.registers.f.zero,
                    self.registers.f.subtract,
                    self.registers.f.half_carry,
                    self.registers.f.carry,
                );
    
                8 // AND n takes 8 cycles
            }

            Instruction::CPImmediate(value) => {
                console_log!("Executing CP {:#04X} with A = {:#04X}", value, self.registers.a);
    
                // Perform the comparison (A - value) and set flags
                let result = self.registers.a.wrapping_sub(value);
    
                // Set flags based on the result
                self.registers.f.zero = self.registers.a == value;        // Set if A == n
                self.registers.f.subtract = true;                         // Always set for CP
                self.registers.f.half_carry = (self.registers.a & 0x0F) < (value & 0x0F); // Borrow from bit 4
                self.registers.f.carry = self.registers.a < value;        // Borrow from bit 7
    
                console_log!(
                    "Result of CP: {:#04X}, Flags - Z:{} N:{} H:{} C:{}",
                    result,
                    self.registers.f.zero,
                    self.registers.f.subtract,
                    self.registers.f.half_carry,
                    self.registers.f.carry,
                );
    
                8 // CP n takes 8 cycles
            }


            Instruction::LDImmediate16(RegisterPair::HL, value) => {
                console_log!("Executing LD HL, {:#04X}", value);
                self.registers.set_hl(value);
                12 // LD HL, nn takes 12 cycles
            }

            Instruction::LDImmediate8(ArithmeticTarget::C, value) => {
                console_log!("Executing LD C, {:#04X}", value);
                self.registers.c = value;
                8 // LD C, n takes 8 cycles
            }
            Instruction::JP(address) => {
                self.pc = address;
                16
            }
            Instruction::RET => {
                let low = self.pop_stack();
                let high = self.pop_stack();
                self.pc = ((high as u16) << 8) | low as u16;
                16
            }
            Instruction::CALL(address) => {
                let pc_high = (self.pc >> 8) as u8;
                let pc_low = self.pc as u8;
                self.push_stack(pc_high as u16);
                self.push_stack(pc_low as u16);
                self.pc = address;
                24
            }
            Instruction::LDIOOffsetFromA(offset) => {
                let address = 0xFF00 + offset as u16;
                self.bus.write_byte(address, self.registers.a);
                12
            }
            Instruction::LDIOOffsetToA(offset) => {
                let address = 0xFF00 + offset as u16;
                self.registers.a = self.bus.read_byte(address);
                12
            }
            Instruction::CPImmediate(value) => {
                let result = self.registers.a.wrapping_sub(value);
                self.registers.f.zero = self.registers.a == value;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (self.registers.a & 0x0F) < (value & 0x0F);
                self.registers.f.carry = self.registers.a < value;
                8
            }
            Instruction::DI => {
                self.ime = false;
                4
            }
            Instruction::EI => {
                self.ime = true;
                4
            }
    
            Instruction::EI => { self.ime = true; 4 },
            Instruction::DI => { self.ime = false; 4 },
            Instruction::NOP => { 4 },
            Instruction::JP(addr) => {
                self.pc = addr;
                16
            }
            Instruction::LDHLADecrement => {
                // Store the value in register A to the address pointed by HL
                let address = self.registers.get_hl();
                console_log!("Executing LD (HL-), A. Address: {:#04X}, Value: {:#04X}", address, self.registers.a);
    
                self.bus.write_byte(address, self.registers.a);
    
                // Decrement HL after the write
                let new_hl = address.wrapping_sub(1);
                self.registers.set_hl(new_hl);
                console_log!("Decremented HL to: {:#04X}", new_hl);
    
                8 // LD (HL-), A takes 8 cycles
            }
            Instruction::LDSPImmediate16(value) => {
                console_log!("Executing LD SP, {:#04X}", value);
                self.sp = value;
                12 // LD SP, nn takes 12 cycles
            }
            Instruction::LD(ArithmeticTarget::A, ArithmeticTarget::H) => {
                console_log!("Executing LD A, H");
                self.registers.a = self.registers.h;
                4 // LD A, H takes 4 cycles
            }
            Instruction::JR(offset) => {
                console_log!("Executing JR {:+}", offset);
                
                // Calculate the new program counter value with the offset
                self.pc = self.pc.wrapping_add(offset as u16);
    
                12 // JR n takes 12 cycles
            }
    
            Instruction::PUSH(RegisterPair::HL) => {
                console_log!("Executing PUSH HL");
    
                // Get the high and low bytes of HL
                let hl = self.registers.get_hl();
                let high_byte = (hl >> 8) as u8;
                let low_byte = (hl & 0xFF) as u8;
    
                // Push the high byte onto the stack
                self.sp = self.sp.wrapping_sub(1);
                self.bus.write_byte(self.sp, high_byte);
                console_log!("Pushed high byte {:#04X} to stack at address {:#04X}", high_byte, self.sp);
    
                // Push the low byte onto the stack
                self.sp = self.sp.wrapping_sub(1);
                self.bus.write_byte(self.sp, low_byte);
                console_log!("Pushed low byte {:#04X} to stack at address {:#04X}", low_byte, self.sp);
    
                16 // PUSH nn takes 16 cycles
            }
            Instruction::POP(RegisterPair::HL) => {
                console_log!("Executing POP HL");
    
                // Read the low byte from the stack into register L
                let low_byte = self.bus.read_byte(self.sp);
                self.sp = self.sp.wrapping_add(1);
                self.registers.l = low_byte;
                console_log!("Popped low byte {:#04X} from stack to register L", low_byte);
    
                // Read the high byte from the stack into register H
                let high_byte = self.bus.read_byte(self.sp);
                self.sp = self.sp.wrapping_add(1);
                self.registers.h = high_byte;
                console_log!("Popped high byte {:#04X} from stack to register H", high_byte);
    
                12 // POP nn takes 12 cycles
            }
    
            Instruction::PUSHAF => {
                console_log!("Executing PUSH AF");
    
                // Get the high and low bytes of the AF register pair
                let high_byte = self.registers.a;
                let low_byte = u8::from(self.registers.f);
    
                // Push the high byte (A) onto the stack
                self.sp = self.sp.wrapping_sub(1);
                self.bus.write_byte(self.sp, high_byte);
                console_log!("Pushed high byte {:#04X} to stack at address {:#04X}", high_byte, self.sp);
    
                // Push the low byte (F) onto the stack
                self.sp = self.sp.wrapping_sub(1);
                self.bus.write_byte(self.sp, low_byte);
                console_log!("Pushed low byte {:#04X} to stack at address {:#04X}", low_byte, self.sp);
    
                16 // PUSH AF takes 16 cycles
            }
            Instruction::INCHL => {
                console_log!("Executing INC HL");
                
                // Increment the HL register pair
                let hl = self.registers.get_hl();
                let new_hl = hl.wrapping_add(1);
                self.registers.set_hl(new_hl);
                
                console_log!("Incremented HL to {:#06X}", new_hl);
    
                8 // INC HL takes 8 cycles
            }
            Instruction::LDAFromHLIncrement => {
                console_log!("Executing LD A, (HL+)");
    
                // Get the address from the HL register
                let address = self.registers.get_hl();
    
                // Load the value from memory into register A
                let value = self.bus.read_byte(address);
                self.registers.a = value;
                console_log!("Loaded value {:#04X} from address {:#06X} into A", value, address);
    
                // Increment the HL register by 1
                let new_hl = address.wrapping_add(1);
                self.registers.set_hl(new_hl);
                console_log!("Incremented HL to {:#06X}", new_hl);
    
                8 // LD A, (HL+) takes 8 cycles
            }
            Instruction::POPAF => {
                console_log!("Executing POP AF");
    
                // Read the low byte from the stack into the F register
                let low_byte = self.bus.read_byte(self.sp);
                self.sp = self.sp.wrapping_add(1);
                self.registers.f = FlagRegister::from(low_byte);
                console_log!("Popped low byte (flags) {:#04X} from stack to F", low_byte);
    
                // Read the high byte from the stack into the A register
                let high_byte = self.bus.read_byte(self.sp);
                self.sp = self.sp.wrapping_add(1);
                self.registers.a = high_byte;
                console_log!("Popped high byte {:#04X} from stack to A", high_byte);
    
                12 // POP AF takes 12 cycles
            }
            Instruction::PUSH(RegisterPair::BC) => {
                console_log!("Executing PUSH BC");
    
                // Get the high and low bytes of the BC register pair
                let high_byte = self.registers.b;
                let low_byte = self.registers.c;
    
                // Push the high byte onto the stack
                self.sp = self.sp.wrapping_sub(1);
                self.bus.write_byte(self.sp, high_byte);
                console_log!("Pushed high byte {:#04X} to stack at address {:#06X}", high_byte, self.sp);
    
                // Push the low byte onto the stack
                self.sp = self.sp.wrapping_sub(1);
                self.bus.write_byte(self.sp, low_byte);
                console_log!("Pushed low byte {:#04X} to stack at address {:#06X}", low_byte, self.sp);
    
                16 // PUSH nn takes 16 cycles
            }
            Instruction::INCBC => {
                console_log!("Executing INC BC");
    
                // Increment the BC register pair
                let bc = self.registers.get_bc();
                let new_bc = bc.wrapping_add(1);
                self.registers.set_bc(new_bc);
    
                console_log!("Incremented BC to {:#06X}", new_bc);
    
                8 // INC BC takes 8 cycles
            }
            Instruction::JRZ(offset) => {
                console_log!("Executing JR Z, {:+}", offset);
    
                if self.registers.f.zero {
                    console_log!("Zero flag is set. Jumping to offset {:+}", offset);
                    self.pc = self.pc.wrapping_add(offset as u16);
                    12 // If the jump is taken, it takes 12 cycles
                } else {
                    console_log!("Zero flag not set. Skipping jump");
                    8 // If the jump is not taken, it takes 8 cycles
                }
            }
            Instruction::JRNZ(offset) => {
                console_log!("Executing JR NZ, {:+}", offset);
    
                if !self.registers.f.zero {
                    console_log!("Zero flag not set. Jumping to offset {:+}", offset);
                    self.pc = self.pc.wrapping_add(offset as u16);
                    12 // If the jump is taken, it takes 12 cycles
                } else {
                    console_log!("Zero flag is set. Skipping jump");
                    8 // If the jump is not taken, it takes 8 cycles
                }
            }
    
    
            _ => { 4 },
        }
    }
    
    fn set_register_pair(&mut self, pair: RegisterPair, value: u16) {
        match pair {
            RegisterPair::BC => self.registers.set_bc(value),
            RegisterPair::DE => self.registers.set_de(value),
            RegisterPair::HL => self.registers.set_hl(value),
        }
    }

    // ADD A, register
    fn add(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let (result, carry) = self.registers.a.overflowing_add(value);
        self.update_flags(result, false, carry, (self.registers.a & 0xF) + (value & 0xF) > 0xF);
        self.registers.a = result;
    }

    fn add_hl(&mut self) {
        let hl = self.registers.get_hl();
        let bc = self.registers.get_bc();
        let (result, carry) = hl.overflowing_add(bc);
        self.registers.set_hl(result);
        self.update_flags(0, false, carry, ((hl & 0xFFF) + (bc & 0xFFF)) > 0xFFF);
    }

    fn adc(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry = if self.registers.f.carry { 1 } else { 0 };
        let (result, carry_out) = self.registers.a.overflowing_add(value + carry);
        self.update_flags(result, false, carry_out, ((self.registers.a & 0xF) + (value & 0xF) + carry) > 0xF);
        self.registers.a = result;
    }

    fn sub(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let (result, carry) = self.registers.a.overflowing_sub(value);
        self.update_flags(result, true, carry, (self.registers.a & 0xF) < (value & 0xF));
        self.registers.a = result;
    }

    fn sbc(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry = if self.registers.f.carry { 1 } else { 0 };
        let (result, carry_out) = self.registers.a.overflowing_sub(value + carry);
        self.update_flags(result, true, carry_out, (self.registers.a & 0xF) < (value & 0xF) + carry);
        self.registers.a = result;
    }

    fn and(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        self.registers.a &= value;
        self.update_flags(self.registers.a, false, false, true);
    }

    fn or(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        self.registers.a |= value;
        self.update_flags(self.registers.a, false, false, false);
    }

    fn xor(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        self.registers.a ^= value;
        self.update_flags(self.registers.a, false, false, false);
    }

    fn cp(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let (result, carry) = self.registers.a.overflowing_sub(value);
        self.update_flags(result, true, carry, (self.registers.a & 0xF) < (value & 0xF));
    }

    fn inc(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let result = value.wrapping_add(1);
        self.set_register_value(target, result);
        self.update_flags(result, false, false, (value & 0xF) + 1 > 0xF);
    }
    
    fn dec(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let result = value.wrapping_sub(1);
        self.set_register_value(target, result);
        self.update_flags(result, true, false, (value & 0xF) < 1);
    }

    fn ccf(&mut self) {
        self.registers.f.carry = !self.registers.f.carry; // Toggle the carry flag
        self.registers.f.subtract = false; // Clear subtract flag
        self.registers.f.half_carry = false; // Clear half-carry flag
    }

    fn scf(&mut self) {
        self.registers.f.carry = true; // Set the carry flag
        self.registers.f.subtract = false; // Clear subtract flag
        self.registers.f.half_carry = false; // Clear half-carry flag
    }

    fn cpl(&mut self) {
        self.registers.a = !self.registers.a; // Invert all bits of register A
        self.registers.f.subtract = true; // Set subtract flag
        self.registers.f.half_carry = true; // Set half-carry flag
    }
    

    fn rra(&mut self) {
        let carry_in = if self.registers.f.carry { 0x80 } else { 0 };
        let carry_out = (self.registers.a & 0x01) != 0;
        self.registers.a = (self.registers.a >> 1) | carry_in;
        self.update_flags(self.registers.a, false, carry_out, false);
    }

    fn rla(&mut self) {
        let carry_in = if self.registers.f.carry { 1 } else { 0 };
        let carry_out = (self.registers.a & 0x80) != 0;
        self.registers.a = (self.registers.a << 1) | carry_in;
        self.update_flags(self.registers.a, false, carry_out, false);
    }

    fn rrca(&mut self) {
        let carry_out = (self.registers.a & 0x01) != 0;
        self.registers.a = (self.registers.a >> 1) | (self.registers.a << 7);
        self.update_flags(self.registers.a, false, carry_out, false);
    }

    fn rlca(&mut self) {
        let carry_out = (self.registers.a & 0x80) != 0;
        self.registers.a = (self.registers.a << 1) | (self.registers.a >> 7);
        self.update_flags(self.registers.a, false, carry_out, false);
    }
        
    fn bit(&mut self, bit: u8, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        self.registers.f.zero = (value & (1 << bit)) == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
    }
    
    fn set_bit(&mut self, bit: u8, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let result = value | (1 << bit);
        self.set_register_value(target, result);
    }
    
    fn reset_bit(&mut self, bit: u8, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let result = value & !(1 << bit);
        self.set_register_value(target, result);
    }

    fn srl(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry = (value & 0x01) != 0;
        let result = value >> 1;
        self.set_register_value(target, result);
        self.update_flags(result, false, carry, false);
    }

    fn rr(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry_in = if self.registers.f.carry { 0x80 } else { 0 };
        let carry_out = (value & 0x01) != 0;
        let result = (value >> 1) | carry_in;
        self.set_register_value(target, result);
        self.update_flags(result, false, carry_out, false);
    }

    fn rl(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry_in = if self.registers.f.carry { 1 } else { 0 };
        let carry_out = (value & 0x80) != 0;
        let result = (value << 1) | carry_in;
        self.set_register_value(target, result);
        self.update_flags(result, false, carry_out, false);
    }

    fn rrc(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry_out = (value & 0x01) != 0;
        let result = (value >> 1) | (value << 7);
        self.set_register_value(target, result);
        self.update_flags(result, false, carry_out, false);
    }

    fn rlc(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry_out = (value & 0x80) != 0;
        let result = (value << 1) | (value >> 7);
        self.set_register_value(target, result);
        self.update_flags(result, false, carry_out, false);
    }

    fn sra(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry = (value & 0x01) != 0;
        let result = (value >> 1) | (value & 0x80);
        self.set_register_value(target, result);
        self.update_flags(result, false, carry, false);
    }

    fn sla(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let carry = (value & 0x80) != 0;
        let result = value << 1;
        self.set_register_value(target, result);
        self.update_flags(result, false, carry, false);
    }

    fn swap(&mut self, target: ArithmeticTarget) {
        let value = self.get_register_value(target);
        let result = (value >> 4) | (value << 4);
        self.set_register_value(target, result);
        self.update_flags(result, false, false, false);
    }
    
    
    fn get_register_value(&self, target: ArithmeticTarget) -> u8 {
        match target {
            ArithmeticTarget::A => self.registers.a,
            ArithmeticTarget::B => self.registers.b,
            ArithmeticTarget::C => self.registers.c,
            ArithmeticTarget::D => self.registers.d,
            ArithmeticTarget::E => self.registers.e,
            ArithmeticTarget::H => self.registers.h,
            ArithmeticTarget::L => self.registers.l,
        }
    }

    fn set_register_value(&mut self, target: ArithmeticTarget, value: u8) {
        match target {
            ArithmeticTarget::A => self.registers.a = value,
            ArithmeticTarget::B => self.registers.b = value,
            ArithmeticTarget::C => self.registers.c = value,
            ArithmeticTarget::D => self.registers.d = value,
            ArithmeticTarget::E => self.registers.e = value,
            ArithmeticTarget::H => self.registers.h = value,
            ArithmeticTarget::L => self.registers.l = value,
        }
    }

    fn update_flags(&mut self, result: u8, subtract: bool, carry: bool, half_carry: bool) {
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = subtract;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = half_carry;
    }
}
