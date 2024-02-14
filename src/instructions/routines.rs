use crate::cpu::{Cpu, Flag};

use super::Instruction;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Condition {
    Zero,
    NotZero,
    Carry,
    NotCarry,
}

impl From<Condition> for Flag {
    fn from(val: Condition) -> Self {
        match val {
            Condition::Zero => Flag::Zero,
            Condition::NotZero => Flag::Zero,
            Condition::Carry => Flag::Carry,
            Condition::NotCarry => Flag::Carry,
        }
    }
}

impl From<u8> for Condition {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Self::NotZero,
            0b01 => Self::Zero,
            0b10 => Self::NotCarry,
            0b11 => Self::Carry,
            _ => panic!("Invalid condition: {:b}", value),
        }
    }
    // 0b100000 | 0b110000 | 0b101000 | 0b111000
}

pub(crate) type Conditional = Option<Condition>;

pub(crate) struct Call(pub(crate) Conditional, pub(crate) u16);

impl Instruction for Call {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        if self.0.is_some() && !cpu.test_flag(self.0.unwrap().into()) {
            return 12;
        }

        // Push next instruction onto stack
        let pc = *cpu.registers().pc;
        let sp = *cpu.registers().sp;
        cpu.write_u8(sp as usize - 1, (pc >> 8) as u8);
        cpu.write_u8(sp as usize - 2, (pc & 0xff) as u8);
        *cpu.registers_mut().sp -= 2;

        // Jump to address
        *cpu.registers_mut().pc = self.1;

        24
    }
}

pub(crate) enum Jump {
    Internal,
    Immediate(Conditional, u16),
    Relative(Conditional, i8),
}

impl Instruction for Jump {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Self::Internal => {
                *cpu.registers_mut().pc = *cpu.registers().hl;

                4
            }
            Self::Immediate(cond, value) => {
                if cond.is_some() && !cpu.test_flag(cond.unwrap().into()) {
                    return 12;
                }

                *cpu.registers_mut().pc = *value;

                16
            }
            Self::Relative(cond, value) => {
                if cond.is_some() && !cpu.test_flag(cond.unwrap().into()) {
                    return 8;
                }

                let pc = *cpu.registers().pc;
                *cpu.registers_mut().pc = pc.wrapping_add(*value as u16);

                12
            }
        }
    }
}

pub(crate) enum Ret {
    Internal(Conditional),
    EnableInterrupts,
}

impl Instruction for Ret {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Self::Internal(cond) => {
                if cond.is_some() && !cpu.test_flag(cond.unwrap().into()) {
                    return 8;
                }

                let sp = *cpu.registers().sp;
                let pc = cpu.read_u16(sp as usize);
                *cpu.registers_mut().sp += 2;
                *cpu.registers_mut().pc = pc;

                cond.is_some() as usize * 20 + cond.is_none() as usize * 16
            }
            Self::EnableInterrupts => {
                cpu.registers_mut().ime = true;

                let sp = *cpu.registers().sp;
                let pc = cpu.read_u16(sp as usize);
                *cpu.registers_mut().sp += 2;
                *cpu.registers_mut().pc = pc;

                16
            }
        }
    }
}
pub(crate) struct Rst(pub(crate) u8);

impl Instruction for Rst {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        let sp = *cpu.registers().sp;
        let pc = *cpu.registers().pc;
        cpu.write_u8(sp as usize - 1, (pc >> 8) as u8);
        cpu.write_u8(sp as usize - 2, (pc & 0xff) as u8);
        *cpu.registers_mut().sp -= 2;

        *cpu.registers_mut().pc = self.0 as u16;

        16
    }
}
