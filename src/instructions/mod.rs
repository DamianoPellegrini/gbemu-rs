use crate::{
    cpu::{Cpu, Registers},
    memory::Read,
};

use self::loads::LoadDirection;

mod arithmetics;
mod bits;
mod cpu_control;
mod loads;
mod routines;

pub type Register8Source = Register8Index;
pub type Register8Destination = Register8Index;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register8Index {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
    HL,
}

impl Register8Index {
    pub fn set(&self, cpu: &mut dyn Cpu, value: u8) {
        match self {
            Self::A => cpu.registers_mut().af.hi = value,
            Self::B => cpu.registers_mut().bc.hi = value,
            Self::C => cpu.registers_mut().bc.lo = value,
            Self::D => cpu.registers_mut().de.hi = value,
            Self::E => cpu.registers_mut().de.lo = value,
            Self::H => cpu.registers_mut().hl.hi = value,
            Self::L => cpu.registers_mut().hl.lo = value,
            Self::F => cpu.registers_mut().af.lo = value,
            Self::HL => cpu.write_u8(*cpu.registers().hl as usize, value),
        }
    }

    pub fn get(&self, cpu: &dyn Cpu) -> u8 {
        unsafe {
            match self {
                Self::A => cpu.registers().af.hi,
                Self::B => cpu.registers().bc.hi,
                Self::C => cpu.registers().bc.lo,
                Self::D => cpu.registers().de.hi,
                Self::E => cpu.registers().de.lo,
                Self::H => cpu.registers().hl.hi,
                Self::L => cpu.registers().hl.lo,
                Self::F => cpu.registers().af.lo,
                Self::HL => cpu.read_u8(*cpu.registers().hl as usize),
            }
        }
    }
}

impl From<u8> for Register8Index {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Register8Index::B,
            0x1 => Register8Index::C,
            0x2 => Register8Index::D,
            0x3 => Register8Index::E,
            0x4 => Register8Index::H,
            0x5 => Register8Index::L,
            0x6 => Register8Index::HL,
            0x7 => Register8Index::A,
            _ => panic!("Invalid register index: {:#02x}", value),
        }
    }
}

pub type Register16Source = Register16Index;
pub type Register16Destination = Register16Index;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register16Index {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl Register16Index {
    pub fn set(&self, cpu: &mut dyn Cpu, value: u16) {
        match self {
            Self::AF => *cpu.registers_mut().af = value,
            Self::BC => *cpu.registers_mut().bc = value,
            Self::DE => *cpu.registers_mut().de = value,
            Self::HL => *cpu.registers_mut().hl = value,
            Self::SP => *cpu.registers_mut().sp = value,
            Self::PC => *cpu.registers_mut().pc = value,
        }
    }

    pub fn get(&self, cpu: &dyn Cpu) -> u16 {
        match self {
            Self::AF => *cpu.registers().af,
            Self::BC => *cpu.registers().bc,
            Self::DE => *cpu.registers().de,
            Self::HL => *cpu.registers().hl,
            Self::SP => *cpu.registers().sp,
            Self::PC => *cpu.registers().pc,
        }
    }
}

impl From<u8> for Register16Index {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Register16Index::BC,
            0x1 => Register16Index::DE,
            0x2 => Register16Index::HL,
            0x3 => Register16Index::SP,
            _ => panic!("Invalid register index: {:#02x}", value),
        }
    }
}

pub trait Instruction {
    /// ### Execute
    ///
    /// Execute the instruction and return the number of clock-cycles
    /// consumed by the instruction.
    fn execute(&self, cpu: &mut dyn Cpu) -> usize;
}

pub trait Assemble {
    fn assemble(&self) -> Vec<u8>;
}

pub trait InstructionDecoder: Registers + Read {
    fn fetch(&mut self) -> u8 {
        let pc = self.registers().pc;
        *self.registers_mut().pc += 1;
        self.read_u8(*pc as usize)
    }

    fn decode(&mut self, opcode: u8) -> Box<dyn Instruction> {
        match opcode {
            // == Misc/Control ==
            0x0 => Box::new(cpu_control::Nop),
            0x10 => Box::new(cpu_control::Stop),
            0x76 => Box::new(cpu_control::Halt),
            0xF3 => Box::new(cpu_control::Di),
            0xFB => Box::new(cpu_control::Ei),

            // == Jump/Routines ==

            // JR
            0x18 => Box::new(routines::Jump::Relative(None, self.fetch() as i8)),

            // JR cond
            // 0b100000 | 0b110000 | 0b101000 | 0b111000
            0x20 | 0x30 | 0x28 | 0x38 => Box::new(routines::Jump::Relative(
                Some(routines::Condition::from((opcode >> 3) & 0b11)),
                self.fetch() as i8,
            )),

            // JP
            0xC3 => Box::new(routines::Jump::Immediate(
                None,
                self.fetch() as u16 | ((self.fetch() as u16) << 8),
            )),

            // JP cond
            0xC2 | 0xD2 | 0xCA | 0xDA => Box::new(routines::Jump::Immediate(
                Some(routines::Condition::from((opcode >> 3) & 0b11)),
                self.fetch() as u16 | ((self.fetch() as u16) << 8),
            )),

            // JP HL
            0xE9 => Box::new(routines::Jump::Internal),

            // Call
            0xCD => Box::new(routines::Call(
                None,
                self.fetch() as u16 | ((self.fetch() as u16) << 8),
            )),

            // Call cond
            0xC4 | 0xD4 | 0xCC | 0xDC => Box::new(routines::Call(
                Some(routines::Condition::from((opcode >> 3) & 0b11)),
                self.fetch() as u16 | ((self.fetch() as u16) << 8),
            )),

            // Ret
            0xC9 => Box::new(routines::Ret::Internal(None)),

            // Ret cond
            0xC0 | 0xD0 | 0xC8 | 0xD8 => Box::new(routines::Ret::Internal(Some(
                routines::Condition::from((opcode >> 3) & 0b11),
            ))),

            // Reti
            0xD9 => Box::new(routines::Ret::EnableInterrupts),

            // Rst
            0xCF | 0xDF | 0xEF | 0xFF => Box::new(routines::Rst(opcode & 0b00110000 | 0x08)),

            // == Arithmetic/Logic ==

            // Adds
            0x80..=0x87 => Box::new(arithmetics::Add::Internal(Register8Index::from(
                opcode & 0b111,
            ))),
            0xC6 => Box::new(arithmetics::Add::Immediate(self.fetch())),
            0x9 | 0x19 | 0x29 | 0x39 => Box::new(arithmetics::Add::Internal16(
                Register16Index::from((opcode >> 4) & 0b11),
            )),
            0xE8 => Box::new(arithmetics::Add::StackPointer(self.fetch() as i8)),

            // Adc
            0x88..=0x8F => Box::new(arithmetics::Adc::Internal(Register8Index::from(
                opcode & 0b111,
            ))),
            0xCE => Box::new(arithmetics::Adc::Immediate(self.fetch())),

            // Sub
            0x90..=0x97 => Box::new(arithmetics::Sub::Internal(Register8Index::from(
                opcode & 0b111,
            ))),
            0xD6 => Box::new(arithmetics::Sub::Immediate(self.fetch())),

            // Sbc
            0x98..=0x9F => Box::new(arithmetics::Sbc::Internal(Register8Index::from(
                opcode & 0b111,
            ))),
            0xDE => Box::new(arithmetics::Sbc::Immediate(self.fetch())),

            // And
            0xA0..=0xA7 => Box::new(arithmetics::And::Internal(Register8Index::from(
                opcode & 0b111,
            ))),
            0xE6 => Box::new(arithmetics::And::Immediate(self.fetch())),

            // Xor
            0xA8..=0xAF => Box::new(arithmetics::Xor::Internal(Register8Index::from(
                opcode & 0b111,
            ))),
            0xEE => Box::new(arithmetics::Xor::Immediate(self.fetch())),

            // Or
            0xB0..=0xB7 => Box::new(arithmetics::Or::Internal(Register8Index::from(
                opcode & 0b111,
            ))),
            0xF6 => Box::new(arithmetics::Or::Immediate(self.fetch())),

            // Cp
            0xB8..=0xBF => Box::new(arithmetics::Cp::Internal(Register8Index::from(
                opcode & 0b111,
            ))),
            0xFE => Box::new(arithmetics::Cp::Immediate(self.fetch())),

            // Inc
            0x4 | 0xC | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => Box::new(
                arithmetics::Inc::Internal(Register8Index::from((opcode >> 3) & 0b111)),
            ),
            0x3 | 0x13 | 0x23 | 0x33 => Box::new(arithmetics::Inc::Internal16(
                Register16Index::from((opcode >> 4) & 0b11),
            )),

            // Dec
            0x5 | 0xD | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => Box::new(
                arithmetics::Dec::Internal(Register8Index::from((opcode >> 3) & 0b111)),
            ),
            0xB | 0x1B | 0x2B | 0x3B => Box::new(arithmetics::Dec::Internal16(
                Register16Index::from((opcode >> 4) & 0b11),
            )),

            // Daa
            0x27 => Box::new(arithmetics::Daa),

            // Cpl
            0x2F => Box::new(arithmetics::Cpl),

            // Scf
            0x37 => Box::new(arithmetics::Scf),

            // Ccf
            0x3F => Box::new(arithmetics::Ccf),

            // == Loads/Stack ==

            // == Load8 ==

            // LD r8, r8 Internal
            0x40..=0x6F | 0x70..=0x75 | 0x77..=0x7F => Box::new(loads::Load8::Internal(
                Register8Index::from(opcode & 0b111),
                Register8Index::from((opcode >> 3) & 0b111),
            )),

            // LD r8, n8 Immediate
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => Box::new(
                loads::Load8::Immediate(Register8Index::from((opcode >> 3) & 0b111), self.fetch()),
            ),

            // LD [C], A
            0xE2 => Box::new(loads::Load8::CPointer(LoadDirection::Into)),
            // LD A, [C]
            0xF2 => Box::new(loads::Load8::CPointer(LoadDirection::From)),

            // LD [n8], A
            0xE0 => Box::new(loads::Load8::ImmediatePointer(
                self.fetch(),
                LoadDirection::Into,
            )),
            // LD A, [n8]
            0xF0 => Box::new(loads::Load8::ImmediatePointer(
                self.fetch(),
                LoadDirection::From,
            )),

            // LD [r16], A
            0x02 | 0x12 => Box::new(loads::Load8::InternalPointer(
                Register16Index::from((opcode >> 4) & 0b11),
                LoadDirection::Into,
                None,
            )),
            // LD A, [r16]
            0x0A | 0x1A => Box::new(loads::Load8::InternalPointer(
                Register16Index::from((opcode >> 4) & 0b11),
                LoadDirection::From,
                None,
            )),

            // LD [HL+], A
            0x22 => Box::new(loads::Load8::InternalPointer(
                Register16Index::from((opcode >> 4) & 0b11),
                LoadDirection::Into,
                Some(true),
            )),
            // LD A, [HL+]
            0x2A => Box::new(loads::Load8::InternalPointer(
                Register16Index::from((opcode >> 4) & 0b11),
                LoadDirection::From,
                Some(true),
            )),

            // LD [HL-], A
            0x32 => Box::new(loads::Load8::InternalPointer(
                Register16Index::from((opcode >> 4) & 0b11),
                LoadDirection::Into,
                Some(false),
            )),
            // LD A, [HL-]
            0x3A => Box::new(loads::Load8::InternalPointer(
                Register16Index::from((opcode >> 4) & 0b11),
                LoadDirection::From,
                Some(false),
            )),

            // LD [a16], A
            0xEA => Box::new(loads::Load8::ImmediateMemory(
                self.fetch() as u16 | ((self.fetch() as u16) << 8),
                LoadDirection::Into,
            )),
            // LD A, [a16]
            0xFA => Box::new(loads::Load8::ImmediateMemory(
                self.fetch() as u16 | ((self.fetch() as u16) << 8),
                LoadDirection::From,
            )),

            // == Load16 ==

            // LD r16, n16 Immediate
            0x01 | 0x11 | 0x21 | 0x31 => Box::new(loads::Load16::Immediate(
                Register16Index::from((opcode >> 4) & 0b11),
                self.fetch() as u16 | ((self.fetch() as u16) << 8),
            )),

            // LD SP, HL
            0xF9 => Box::new(loads::Load16::StackHL(None)),
            // LD HL, SP+e8
            0xF8 => Box::new(loads::Load16::StackHL(Some(self.fetch() as i8))),

            // LD [n16], SP
            0x08 => Box::new(loads::Load16::StackToMemory(
                self.fetch() as u16 | ((self.fetch() as u16) << 8),
            )),

            // PUSH
            0xC5 | 0xD5 | 0xE5 | 0xF5 => Box::new(loads::Load16::Push(Register16Index::from(
                (opcode >> 4) & 0b11,
            ))),

            // POP
            0xC1 | 0xD1 | 0xE1 | 0xF1 => Box::new(loads::Load16::Pop(Register16Index::from(
                (opcode >> 4) & 0b11,
            ))),

            // == Prefixed ==
            0xCB => match self.fetch() {
                // RLC
                0x00..=0x07 => Box::new(bits::Rotate::LeftCarry(Register8Index::from(
                    opcode & 0b111,
                ))),

                // RRC
                0x08..=0x0E => Box::new(bits::Rotate::RightCarry(Register8Index::from(
                    opcode & 0b111,
                ))),

                // RL
                0x10..=0x17 => Box::new(bits::Rotate::Left(Register8Index::from(opcode & 0b111))),

                // RR
                0x18..=0x1F => Box::new(bits::Rotate::Right(Register8Index::from(opcode & 0b111))),

                // SLA
                0x20..=0x27 => Box::new(bits::Shift::Left(Register8Index::from(opcode & 0b111))),

                // SRA
                0x28..=0x2F => Box::new(bits::Shift::Right(Register8Index::from(opcode & 0b111))),

                // Swap
                0x30..=0x37 => Box::new(bits::Swap(Register8Index::from(opcode & 0b111))),

                // SRL
                0x38..=0x3F => Box::new(bits::Shift::RightLogically(Register8Index::from(
                    opcode & 0b111,
                ))),

                // Bit
                0x40..=0x7F => Box::new(bits::Bit::Test(
                    (opcode & 0b111) >> 3,
                    Register8Index::from(opcode & 0b111),
                )),

                // Res
                0x80..=0xBF => Box::new(bits::Bit::Reset(
                    (opcode & 0b111) >> 3,
                    Register8Index::from(opcode & 0b111),
                )),

                // Set
                0xC0..=0xFF => Box::new(bits::Bit::Set(
                    (opcode & 0b111) >> 3,
                    Register8Index::from(opcode & 0b111),
                )),

                _ => panic!(
                    "Unimplemented prefixed opcode: {:#04x}",
                    0xCB00 | opcode as u16
                ),
            },

            _ => panic!("Unimplemented opcode: {:#02x}", opcode),
        }
    }
}

impl Iterator for dyn InstructionDecoder {
    type Item = Box<dyn Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
        let opcode = self.fetch();
        Some(self.decode(opcode))
    }
}
