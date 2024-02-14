use crate::cpu::{Cpu, Flag};

use super::{Instruction, Register8Index};

pub(crate) type BitIndex = u8;

pub(crate) enum Bit {
    Set(BitIndex, Register8Index),
    Reset(BitIndex, Register8Index),
    Test(BitIndex, Register8Index),
}

impl Instruction for Bit {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Self::Set(bit, dst) => {
                let value = dst.get(cpu);
                dst.set(cpu, value | (1 << bit));

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL) as usize * 8
            }
            Self::Reset(bit, dst) => {
                let value = dst.get(cpu);
                dst.set(cpu, value & !(1 << bit));

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL) as usize * 8
            }
            Self::Test(bit, dst) => {
                let value = dst.get(cpu);
                let result = value & (1 << bit);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, true);

                (*dst == Register8Index::HL) as usize * 12
                    + (*dst != Register8Index::HL) as usize * 8
            }
        }
    }
}

pub(crate) struct Swap(pub(crate) Register8Index);

impl Instruction for Swap {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        let value = self.0.get(cpu);
        let result = (value << 4) | (value >> 4);
        self.0.set(cpu, result);

        (self.0 == Register8Index::HL) as usize * 16 + (self.0 != Register8Index::HL) as usize * 8
    }
}

pub(crate) enum Rotate {
    Left(Register8Index),
    LeftCarry(Register8Index),
    Right(Register8Index),
    RightCarry(Register8Index),
}

impl Instruction for Rotate {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Self::Left(dst) => {
                let value = dst.get(cpu);
                let result = value.rotate_left(1);
                dst.set(cpu, result);
                cpu.set_flag(Flag::Zero, result == 0 && *dst != Register8Index::A);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, false);
                cpu.set_flag(Flag::Carry, value & 0x80 != 0);

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL && *dst != Register8Index::A) as usize * 8
                    + (*dst == Register8Index::A) as usize * 4
            }
            Self::LeftCarry(dst) => {
                let value = dst.get(cpu);
                let result = (value << 1) | (cpu.test_flag(Flag::Carry) as u8);
                dst.set(cpu, result);
                cpu.set_flag(Flag::Zero, result == 0 && *dst != Register8Index::A);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, false);
                cpu.set_flag(Flag::Carry, value & 0x80 != 0);

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL && *dst != Register8Index::A) as usize * 8
                    + (*dst == Register8Index::A) as usize * 4
            }
            Self::Right(dst) => {
                let value = dst.get(cpu);
                let result = value.rotate_right(1);
                dst.set(cpu, result);
                cpu.set_flag(Flag::Zero, result == 0 && *dst != Register8Index::A);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, false);
                cpu.set_flag(Flag::Carry, value & 0x01 != 0);

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL && *dst != Register8Index::A) as usize * 8
                    + (*dst == Register8Index::A) as usize * 4
            }
            Self::RightCarry(dst) => {
                let value = dst.get(cpu);
                let result = (value >> 1) | ((cpu.test_flag(Flag::Carry) as u8) << 7);
                dst.set(cpu, result);
                cpu.set_flag(Flag::Zero, result == 0 && *dst != Register8Index::A);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, false);
                cpu.set_flag(Flag::Carry, value & 0x01 != 0);

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL && *dst != Register8Index::A) as usize * 8
                    + (*dst == Register8Index::A) as usize * 4
            }
        }
    }
}

pub(crate) enum Shift {
    Left(Register8Index),
    LeftLogically(Register8Index),
    Right(Register8Index),
    RightLogically(Register8Index),
}

impl Instruction for Shift {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Self::Left(dst) => {
                let value = dst.get(cpu);
                let result = value << 1;
                dst.set(cpu, result);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, false);
                cpu.set_flag(Flag::Carry, value & 0x80 != 0);

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL && *dst != Register8Index::A) as usize * 8
                    + (*dst == Register8Index::A) as usize * 4
            }
            Self::LeftLogically(dst) => {
                let value = dst.get(cpu);
                let result = value << 1;
                dst.set(cpu, result);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, false);
                cpu.set_flag(Flag::Carry, value & 0x80 != 0);

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL && *dst != Register8Index::A) as usize * 8
                    + (*dst == Register8Index::A) as usize * 4
            }
            Self::Right(dst) => {
                let value = dst.get(cpu);
                let result = value >> 1;
                dst.set(cpu, result);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, false);
                cpu.set_flag(Flag::Carry, value & 0x01 != 0);

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL && *dst != Register8Index::A) as usize * 8
                    + (*dst == Register8Index::A) as usize * 4
            }
            Self::RightLogically(dst) => {
                let value = dst.get(cpu);
                let result = (value as i8 >> 1) as u8;
                dst.set(cpu, result);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::HalfCarry, false);
                cpu.set_flag(Flag::Carry, value & 0x01 != 0);

                (*dst == Register8Index::HL) as usize * 16
                    + (*dst != Register8Index::HL && *dst != Register8Index::A) as usize * 8
                    + (*dst == Register8Index::A) as usize * 4
            }
        }
    }
}
