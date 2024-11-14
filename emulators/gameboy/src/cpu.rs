
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
}

#[derive(Clone, Copy)]
enum ArithmeticTarget {
    A, B, C, D, E, H, L,
}

struct MemoryBus {
    memory: [u8; 0xFFFF]
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    
}

struct CPU {
    registers: Registers,
    pc: u16,
    bus: MemoryBus

}

impl CPU {
    fn step(&mut self) {
        let opcode = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);

    }

    fn decode_opcode(&mut self, opcode: u8) -> Instruction {
        match opcode {
            // NOP (No Operation)
            //0x00 => Instruction::NOP,

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

            // Halt (stop CPU until an interrupt occurs)
            //0x76 => Instruction::HALT,

            // Extended opcodes (0xCB prefix)
            0xCB => self.decode_extended_opcode(),

            _ => panic!("Unknown opcode: 0x{:02X}", opcode),
        }
    }

    /// Decodes extended opcodes (0xCB-prefixed).
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

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(target) => self.add(target),
            Instruction::ADDHL => self.add_hl(),
            Instruction::ADC(target) => self.adc(target),
            Instruction::SUB(target) => self.sub(target),
            Instruction::SBC(target) => self.sbc(target),
            Instruction::AND(target) => self.and(target),
            Instruction::OR(target) => self.or(target),
            Instruction::XOR(target) => self.xor(target),
            Instruction::CP(target) => self.cp(target),
            Instruction::INC(target) => self.inc(target),
            Instruction::DEC(target) => self.dec(target),
            Instruction::CCF => self.ccf(),
            Instruction::SCF => self.scf(),
            Instruction::RRA => self.rra(),
            Instruction::RLA => self.rla(),
            Instruction::RRCA => self.rrca(),
            Instruction::RLCA => self.rlca(),
            Instruction::CPL => self.cpl(),
            Instruction::BIT(bit, target) => self.bit(bit, target),
            Instruction::RESET(bit, target) => self.reset_bit(bit, target),
            Instruction::SET(bit, target) => self.set_bit(bit, target),
            Instruction::SRL(target) => self.srl(target),
            Instruction::RR(target) => self.rr(target),
            Instruction::RL(target) => self.rl(target),
            Instruction::RRC(target) => self.rrc(target),
            Instruction::RLC(target) => self.rlc(target),
            Instruction::SRA(target) => self.sra(target),
            Instruction::SLA(target) => self.sla(target),
            Instruction::SWAP(target) => self.swap(target),
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
