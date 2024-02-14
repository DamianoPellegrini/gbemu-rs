use crate::cpu::{Cpu, Flag};

use super::{Instruction, Register16Index, Register8Destination, Register8Index, Register8Source};

// Load internal
// LD r, r   0b01xxxyyy        | 0b01000000..=0b01111111
// LD r, (HL) 0b01xxx110       | 0b01000110..=0b01111110
// LD (HL), r 0b01110yyy       | 0b01110000..=0b01110111

// Load immediate
// LD r, imm 0b00xxx110 + 0x00 | 0b00000110..=0b00111110
// LD (HL), imm 0b00110110 + 0x00

// Load from address register
// LD A, (BC) 0b00001010
// LD A, (DE) 0b00011010
// LD (BC), A 0b00000010
// LD (DE), A 0b00010010

// Load From Immediate address
// LD A, (nn) 0b11111010 + 0x0000
// LD (nn), A 0b11101010 + 0x0000

// Load Pointers
// LDH A, (FF00 + C) 0b11110010
// LDH (FF00 + C), A 0b11100010
// LDH A, (FF00 + n) 0b11110000 + 0x00
// LDH (FF00 + n), A 0b11100000 + 0x00

// Loads Increment/Decrement
// LD A, (HL-) 0b00111010
// LD (HL-), A 0b00110010
// LD A, (HL+) 0b00101010
// LD (HL+), A 0b00100010

pub(crate) enum LoadDirection {
    From,
    Into,
}

pub(crate) enum Load8 {
    /// Loads the value from one 8-bit register into another.
    Internal(Register8Source, Register8Destination),
    /// Loads immediates bytes into an 8-bit register.
    Immediate(Register8Destination, u8),
    /// Loads the value from or into A into or from an immediate 16-bit address.
    ImmediateMemory(u16, LoadDirection),
    /// Loads the value from or into 0xFF00 + an 8-bit immediate offset into or from A.
    ImmediatePointer(u8, LoadDirection),
    /// Loads the value from or into A into or from the address stored in a 16-bit register.
    InternalPointer(Register16Index, LoadDirection, Option<bool>),
    /// Loads the value from or into A into or from 0xFF00 + C.
    CPointer(LoadDirection),
}

impl Instruction for Load8 {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Self::Internal(src, dst) => {
                let value = src.get(cpu);
                dst.set(cpu, value);

                (*dst == Register8Index::HL) as usize * 8
                    + (*dst != Register8Index::HL) as usize * 4
            }
            Self::Immediate(dst, value) => {
                dst.set(cpu, *value);

                (*dst == Register8Index::HL) as usize * 12
                    + (*dst != Register8Index::HL) as usize * 8
            }
            Self::ImmediateMemory(addr, dir) => {
                match dir {
                    LoadDirection::From => {
                        let value = cpu.read_u8(*addr as usize);
                        cpu.registers_mut().af.hi = value;
                    }
                    LoadDirection::Into => {
                        let value = unsafe { cpu.registers().af.hi };
                        cpu.write_u8(*addr as usize, value);
                    }
                }

                16
            }
            Self::ImmediatePointer(offset, dir) => {
                match dir {
                    LoadDirection::From => {
                        let value = cpu.read_u8(0xFF00 + *offset as usize);
                        cpu.registers_mut().af.hi = value;
                    }
                    LoadDirection::Into => {
                        let value = unsafe { cpu.registers().af.hi };
                        cpu.write_u8(0xFF00 + *offset as usize, value);
                    }
                }

                12
            }
            Self::InternalPointer(reg, dir, incdec) => {
                let addr = reg.get(cpu);
                match dir {
                    LoadDirection::From => {
                        let value = cpu.read_u8(addr as usize);
                        cpu.registers_mut().af.hi = value;
                    }
                    LoadDirection::Into => {
                        let value = unsafe { cpu.registers().af.hi };
                        cpu.write_u8(addr as usize, value);
                    }
                }

                if let Some(incdec) = incdec {
                    if *incdec {
                        reg.set(cpu, addr.wrapping_add(1))
                    } else {
                        reg.set(cpu, addr.wrapping_sub(1))
                    }
                }

                8
            }
            Self::CPointer(dir) => {
                match dir {
                    LoadDirection::From => {
                        let value = cpu.read_u8(0xff00 + unsafe { cpu.registers().bc.lo } as usize);
                        cpu.registers_mut().af.hi = value;
                    }
                    LoadDirection::Into => {
                        let value = unsafe { cpu.registers().af.hi };
                        cpu.write_u8(0xff00 + unsafe { cpu.registers().bc.lo } as usize, value);
                    }
                }

                8
            }
        }
    }
}

pub(crate) enum Load16 {
    Immediate(Register16Index, u16),
    StackToMemory(u16),
    StackHL(Option<i8>),
    Push(Register16Index),
    Pop(Register16Index),
}

impl Instruction for Load16 {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Self::Immediate(dst, value) => {
                dst.set(cpu, *value);

                12
            }
            Self::StackToMemory(addr) => {
                let sp = *cpu.registers().sp;
                cpu.write_u16(*addr as usize, sp);

                20
            }
            Self::StackHL(offset) => match offset {
                Some(offset) => {
                    let sp = *cpu.registers().sp;
                    *cpu.registers_mut().hl = sp + *offset as u16;
                    12
                }
                None => {
                    *cpu.registers_mut().sp = *cpu.registers().hl;
                    8
                }
            },
            Self::Push(src) => {
                let sp = *cpu.registers().sp;
                let value = src.get(cpu);
                if *src == Register16Index::AF {
                    cpu.write_u16(
                        sp as usize - 2,
                        value & 0xFF00
                            | (if cpu.test_flag(Flag::Zero) { 1 } else { 0 } << 7)
                            | (if cpu.test_flag(Flag::Subtract) { 1 } else { 0 } << 6)
                            | (if cpu.test_flag(Flag::HalfCarry) { 1 } else { 0 } << 5)
                            | (if cpu.test_flag(Flag::Carry) { 1 } else { 0 } << 4),
                    );
                } else {
                    cpu.write_u16(sp as usize - 2, value);
                }
                *cpu.registers_mut().sp -= 2;
                16
            }
            Self::Pop(dst) => {
                if *dst == Register16Index::AF {
                    cpu.set_flag(Flag::Zero, dst.get(cpu) & (1 << 7) != 0);
                    cpu.set_flag(Flag::Subtract, dst.get(cpu) & (1 << 6) != 0);
                    cpu.set_flag(Flag::HalfCarry, dst.get(cpu) & (1 << 5) != 0);
                    cpu.set_flag(Flag::Carry, dst.get(cpu) & (1 << 4) != 0);
                }

                let sp = *cpu.registers().sp;
                let value = cpu.read_u16(sp as usize);
                dst.set(cpu, value);
                *cpu.registers_mut().sp += 2;

                12
            }
        }
    }
}
