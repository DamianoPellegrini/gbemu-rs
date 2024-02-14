use crate::cpu::{Cpu, Flag};

use super::{Instruction, Register16Index, Register8Index};

pub(crate) enum Adc {
    Internal(Register8Index),
    Immediate(u8),
}

impl Instruction for Adc {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Adc::Internal(src) => {
                let value = src.get(cpu);
                let a = Register8Index::A.get(cpu);
                let carry = cpu.test_flag(Flag::Carry) as u8;
                let (result, overflow) = a.overflowing_add(value + carry);
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, overflow);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) + (value & 0x0F) + carry > 0x0F);

                (*src == Register8Index::HL) as usize * 8
                    + (*src != Register8Index::HL) as usize * 4
            }
            Adc::Immediate(value) => {
                let a = Register8Index::A.get(cpu);
                let carry = cpu.test_flag(Flag::Carry) as u8;
                let (result, overflow) = a.overflowing_add(value + carry);
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, overflow);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) + (value & 0x0F) + carry > 0x0F);

                8
            }
        }
    }
}

pub(crate) enum Add {
    Internal(Register8Index),
    Immediate(u8),
    Internal16(Register16Index),
    StackPointer(i8),
}

impl Instruction for Add {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Add::Internal(src) => {
                let value = src.get(cpu);
                let a = Register8Index::A.get(cpu);
                let (result, overflow) = a.overflowing_add(value);
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, overflow);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) + (value & 0x0F) > 0x0F);

                (*src == Register8Index::HL) as usize * 8
                    + (*src != Register8Index::HL) as usize * 4
            }
            Add::Immediate(value) => {
                let a = Register8Index::A.get(cpu);
                let (result, overflow) = a.overflowing_add(*value);
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, overflow);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) + (value & 0x0F) > 0x0F);

                8
            }
            Add::Internal16(src) => {
                let value = src.get(cpu);
                let hl = Register16Index::HL.get(cpu);
                let (result, overflow) = hl.overflowing_add(value);
                Register16Index::HL.set(cpu, result);

                cpu.set_flag(Flag::Zero, false);
                cpu.set_flag(Flag::Carry, overflow);
                cpu.set_flag(Flag::HalfCarry, (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);

                8
            }
            Add::StackPointer(value) => {
                let sp = Register16Index::SP.get(cpu);
                let (result, overflow) = sp.overflowing_add_signed(*value as i16);
                Register16Index::SP.set(cpu, result);

                cpu.set_flag(Flag::Zero, false);
                cpu.set_flag(Flag::Carry, overflow);
                cpu.set_flag(Flag::HalfCarry, (sp & 0x0F) + (*value as u16 & 0x0F) > 0x0F);

                16
            }
        }
    }
}

pub(crate) enum And {
    Internal(Register8Index),
    Immediate(u8),
}

impl Instruction for And {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            And::Internal(src) => {
                let value = src.get(cpu);
                let a = Register8Index::A.get(cpu);
                let result = a & value;
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, false);
                cpu.set_flag(Flag::HalfCarry, true);

                (*src == Register8Index::HL) as usize * 8
                    + (*src != Register8Index::HL) as usize * 4
            }
            And::Immediate(value) => {
                let a = Register8Index::A.get(cpu);
                let result = a & value;
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, false);
                cpu.set_flag(Flag::HalfCarry, true);

                8
            }
        }
    }
}

pub(crate) enum Cp {
    Internal(Register8Index),
    Immediate(u8),
}

impl Instruction for Cp {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Cp::Internal(src) => {
                let value = src.get(cpu);
                let a = Register8Index::A.get(cpu);
                let result = a.wrapping_sub(value);

                cpu.set_flag(Flag::Subtract, true);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, a < value);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) < (value & 0x0F));

                (*src == Register8Index::HL) as usize * 8
                    + (*src != Register8Index::HL) as usize * 4
            }
            Cp::Immediate(value) => {
                let a = Register8Index::A.get(cpu);
                let result = a.wrapping_sub(*value);

                cpu.set_flag(Flag::Subtract, true);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, a < *value);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) < (value & 0x0F));

                8
            }
        }
    }
}

pub(crate) enum Dec {
    Internal(Register8Index),
    Internal16(Register16Index),
}

impl Instruction for Dec {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Dec::Internal(src) => {
                let value = src.get(cpu);
                let (result, _overflow) = value.overflowing_sub(1);
                src.set(cpu, result);

                cpu.set_flag(Flag::Subtract, true);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::HalfCarry, (value & 0x0F) == 0);

                (*src == Register8Index::HL) as usize * 12
                    + (*src != Register8Index::HL) as usize * 4
            }
            Dec::Internal16(src) => {
                let value = src.get(cpu);
                let (result, _overflow) = value.overflowing_sub(1);
                src.set(cpu, result);

                8
            }
        }
    }
}

pub(crate) enum Inc {
    Internal(Register8Index),
    Internal16(Register16Index),
}

impl Instruction for Inc {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Inc::Internal(src) => {
                let value = src.get(cpu);
                let (result, _overflow) = value.overflowing_add(1);
                src.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::HalfCarry, (value & 0x0F) == 0x0F);

                (*src == Register8Index::HL) as usize * 12
                    + (*src != Register8Index::HL) as usize * 4
            }
            Inc::Internal16(src) => {
                let value = src.get(cpu);
                let (result, _overflow) = value.overflowing_add(1);
                src.set(cpu, result);

                8
            }
        }
    }
}

pub(crate) enum Or {
    Internal(Register8Index),
    Immediate(u8),
}

impl Instruction for Or {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Or::Internal(src) => {
                let value = src.get(cpu);
                let a = Register8Index::A.get(cpu);
                let result = a | value;
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, false);
                cpu.set_flag(Flag::HalfCarry, false);

                (*src == Register8Index::HL) as usize * 8
                    + (*src != Register8Index::HL) as usize * 4
            }
            Or::Immediate(value) => {
                let a = Register8Index::A.get(cpu);
                let result = a | value;
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, false);
                cpu.set_flag(Flag::HalfCarry, false);

                8
            }
        }
    }
}

pub(crate) enum Sbc {
    Internal(Register8Index),
    Immediate(u8),
}

impl Instruction for Sbc {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Sbc::Internal(src) => {
                let value = src.get(cpu);
                let a = Register8Index::A.get(cpu);
                let carry = cpu.test_flag(Flag::Carry) as u8;
                let (result, overflow) = a.overflowing_sub(value + carry);
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, true);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, overflow);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) < (value & 0x0F) + carry);

                (*src == Register8Index::HL) as usize * 8
                    + (*src != Register8Index::HL) as usize * 4
            }
            Sbc::Immediate(value) => {
                let a = Register8Index::A.get(cpu);
                let carry = cpu.test_flag(Flag::Carry) as u8;
                let (result, overflow) = a.overflowing_sub(value + carry);
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, true);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, overflow);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) < (value & 0x0F) + carry);

                8
            }
        }
    }
}

pub(crate) enum Sub {
    Internal(Register8Index),
    Immediate(u8),
}

impl Instruction for Sub {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Sub::Internal(src) => {
                let value = src.get(cpu);
                let a = Register8Index::A.get(cpu);
                let result = a.wrapping_sub(value);
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, true);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, a < value);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) < (value & 0x0F));

                (*src == Register8Index::HL) as usize * 8
                    + (*src != Register8Index::HL) as usize * 4
            }
            Sub::Immediate(value) => {
                let a = Register8Index::A.get(cpu);
                let result = a.wrapping_sub(*value);
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, true);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, a < *value);
                cpu.set_flag(Flag::HalfCarry, (a & 0x0F) < (value & 0x0F));

                8
            }
        }
    }
}

pub(crate) enum Xor {
    Internal(Register8Index),
    Immediate(u8),
}

impl Instruction for Xor {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        match self {
            Xor::Internal(src) => {
                let value = src.get(cpu);
                let a = Register8Index::A.get(cpu);
                let result = a ^ value;
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, false);
                cpu.set_flag(Flag::HalfCarry, false);

                (*src == Register8Index::HL) as usize * 8
                    + (*src != Register8Index::HL) as usize * 4
            }
            Xor::Immediate(value) => {
                let a = Register8Index::A.get(cpu);
                let result = a ^ value;
                Register8Index::A.set(cpu, result);

                cpu.set_flag(Flag::Subtract, false);
                cpu.set_flag(Flag::Zero, result == 0);
                cpu.set_flag(Flag::Carry, false);
                cpu.set_flag(Flag::HalfCarry, false);

                8
            }
        }
    }
}

pub(crate) struct Daa;

impl Instruction for Daa {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        let a = Register8Index::A.get(cpu);
        let mut result = a;

        if cpu.test_flag(Flag::Subtract) {
            if cpu.test_flag(Flag::HalfCarry) {
                result = result.wrapping_sub(0x06);
            }
            if cpu.test_flag(Flag::Carry) {
                result = result.wrapping_sub(0x60);
            }
        } else {
            if cpu.test_flag(Flag::HalfCarry) || (result & 0x0F) > 0x09 {
                result = result.wrapping_add(0x06);
            }
            if cpu.test_flag(Flag::Carry) || result > 0x9F {
                result = result.wrapping_add(0x60);
            }
        }

        cpu.set_flag(Flag::Zero, result == 0);
        cpu.set_flag(Flag::HalfCarry, false);
        cpu.set_flag(Flag::Carry, a < result);

        4
    }
}

pub(crate) struct Cpl;

impl Instruction for Cpl {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        let a = Register8Index::A.get(cpu);
        Register8Index::A.set(cpu, !a);

        cpu.set_flag(Flag::Subtract, true);
        cpu.set_flag(Flag::HalfCarry, true);

        4
    }
}

pub(crate) struct Ccf;

impl Instruction for Ccf {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        cpu.set_flag(Flag::Subtract, false);
        cpu.set_flag(Flag::HalfCarry, false);
        cpu.set_flag(Flag::Carry, !cpu.test_flag(Flag::Carry));

        4
    }
}

pub(crate) struct Scf;

impl Instruction for Scf {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        cpu.set_flag(Flag::Subtract, false);
        cpu.set_flag(Flag::HalfCarry, false);
        cpu.set_flag(Flag::Carry, true);

        4
    }
}
