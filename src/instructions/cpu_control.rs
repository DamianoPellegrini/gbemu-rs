use crate::cpu::Cpu;

use super::Instruction;

pub(crate) struct Nop;

impl Instruction for Nop {
    fn execute(&self, _cpu: &mut dyn Cpu) -> usize {
        4
    }
}

pub(crate) struct Di;

impl Instruction for Di {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        let reg = cpu.registers_mut();
        reg.ime = false;

        4
    }
}

pub(crate) struct Ei;

impl Instruction for Ei {
    fn execute(&self, cpu: &mut dyn Cpu) -> usize {
        let reg = cpu.registers_mut();
        reg.ime = true;

        4
    }
}

pub(crate) struct Halt;

impl Instruction for Halt {
    fn execute(&self, _cpu: &mut dyn Cpu) -> usize {
        unimplemented!("Halt instruction not implemented")
    }
}

pub(crate) struct Stop;

impl Instruction for Stop {
    fn execute(&self, _cpu: &mut dyn Cpu) -> usize {
        unimplemented!("Stop instruction not implemented")
    }
}
