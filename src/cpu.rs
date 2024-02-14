use crate::instructions::InstructionDecoder;
use crate::memory::locations;
use crate::memory::Read;
use crate::memory::Write;

/// The clock speed of the CPU in cycles per second
const CPU_CLOCK_SPEED: f64 = 4194304.0;
const SCANLINE_CLOCK_SPEED: f64 = CPU_CLOCK_SPEED / 456.0;

pub enum Interrupt {
    VBlank,
    LCDStat,
    TimerOverflow,
    SerialTranferComplete,
    Joypad,
}

pub enum Flag {
    Zero,
    Subtract,
    HalfCarry,
    Carry,
}

#[derive(Clone, Copy)]
pub union Register {
    pub value: u16,
    pub hi: u8,
    pub lo: u8,
}

impl std::fmt::Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("Register")
                .field("value", &self.value)
                .field("hi", &self.hi)
                .field("lo", &self.lo)
                .finish()
        }
    }
}

impl std::ops::Deref for Register {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.value }
    }
}

impl std::ops::DerefMut for Register {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut self.value }
    }
}

#[derive(Debug, Clone, Copy)]
/// A representation of the Gameboy Classic CPU
pub struct RegisterFile {
    /// Accumulator and Flags Register
    pub af: Register,
    /// BC Register
    pub bc: Register,
    /// DE Register
    pub de: Register,
    /// HL Register
    pub hl: Register,

    /// Stack Pointer
    pub sp: Register,
    /// Program Counter
    pub pc: Register,

    /// Interrupt Master Enable
    pub ime: bool,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self {
            af: Register { value: 0x0000 },
            bc: Register { value: 0x0000 },
            de: Register { value: 0x0000 },
            hl: Register { value: 0x0000 },
            sp: Register { value: 0x0000 },
            pc: Register { value: 0x0000 },
            ime: false,
        }
    }
}

pub trait Registers {
    fn registers(&self) -> &RegisterFile;
    fn registers_mut(&mut self) -> &mut RegisterFile;

    fn set_flag(&mut self, flag: Flag, value: bool) {
        unsafe {
            match flag {
                Flag::Zero => {
                    if value {
                        self.registers_mut().af.lo |= 0b1000_0000;
                    } else {
                        self.registers_mut().af.lo &= 0b0111_1111;
                    }
                }
                Flag::Subtract => {
                    if value {
                        self.registers_mut().af.lo |= 0b0100_0000;
                    } else {
                        self.registers_mut().af.lo &= 0b1011_1111;
                    }
                }
                Flag::HalfCarry => {
                    if value {
                        self.registers_mut().af.lo |= 0b0010_0000;
                    } else {
                        self.registers_mut().af.lo &= 0b1101_1111;
                    }
                }
                Flag::Carry => {
                    if value {
                        self.registers_mut().af.lo |= 0b0001_0000;
                    } else {
                        self.registers_mut().af.lo &= 0b1110_1111;
                    }
                }
            }
        }
    }

    fn test_flag(&self, flag: Flag) -> bool {
        unsafe {
            match flag {
                Flag::Zero => self.registers().af.lo & 0b1000_0000 != 0,
                Flag::Subtract => self.registers().af.lo & 0b0100_0000 != 0,
                Flag::HalfCarry => self.registers().af.lo & 0b0010_0000 != 0,
                Flag::Carry => self.registers().af.lo & 0b0001_0000 != 0,
            }
        }
    }
}

pub trait Cpu: Read + Write + Registers + InstructionDecoder {
    /// Executes clock cycles based on the delta time
    fn tick(&mut self, delta_time: f64)
    where
        Self: Sized,
    {
        let cycles_to_execute = (delta_time * CPU_CLOCK_SPEED) as usize; // TODO: Sum this somewhere to fix sync

        // Instructions execution
        let mut cycles_count = 0;
        loop {
            let opcode = self.fetch();
            let instruction = self.decode(opcode);
            cycles_count += instruction.execute(self);

            // We finished executing the instructions for this tick
            if cycles_count >= cycles_to_execute {
                break;
            }
        }

        // Timers
        let divider_counter = self.read_u8(locations::DIV);
        let divider_ticks = (delta_time * 16384.0) as u64; // TODO: Sum this somewhere to fix sync
        for _ in 0..divider_ticks {
            // Cannot use set_u8 because it would trigger the write memory trap
            self.memory_mut()[locations::DIV] = divider_counter.wrapping_add(1);
        }

        let timer_ctrl = self.read_u8(locations::TAC);
        let timer_enable = timer_ctrl & 0b100 != 0;
        if timer_enable {
            let timer_freq = match timer_ctrl & 0b11 {
                0b00 => 4096.0,
                0b01 => 262144.0,
                0b10 => 65536.0,
                0b11 => 16384.0,
                _ => unreachable!(),
            };
            let timer_ticks = (delta_time * timer_freq) as u64; // TODO: Sum this somewhere to fix sync
            for _ in 0..timer_ticks {
                let timer_counter = self.read_u8(locations::TIMA);

                if timer_counter == 255 {
                    let timer_modulo = self.read_u8(locations::TMA);
                    self.write_u8(locations::TIMA, timer_modulo);
                    self.interrupt(Interrupt::TimerOverflow);
                } else {
                    self.write_u8(locations::TIMA, timer_counter.wrapping_add(1));
                }
            }
        }

        // LCD
        let scanline_ticks = (delta_time * SCANLINE_CLOCK_SPEED) as u64; // TODO: Sum this somewhere to fix sync
        for _ in 0..scanline_ticks {}

        // Interrupts
        if self.registers().ime {
            let interrupt_flag = self.read_u8(locations::IF);
            let interrupt_enable = self.read_u8(locations::IE);

            // There are interrupts to process
            if interrupt_flag > 0 {
                let enabled_interrupts = interrupt_flag & interrupt_enable;
                // In reverse priority order because of the way the interrupts
                // are serviced, first is the highest priority then a RET in the
                // int. handler will service the other interrupts
                for i in (0..5).rev() {
                    // Service i-th interrupt
                    if enabled_interrupts & (1 << i as u8) != 0 {
                        self.registers_mut().ime = false;
                        // Reset bit i of IF
                        self.write_u8(locations::IF, interrupt_flag & !(1 << i));

                        // make a CALL
                        *self.registers_mut().sp -= 2;
                        self.write_u16(*self.registers().sp as usize, *self.registers().pc);

                        match 1 << i as u8 {
                            0b0000_0001 => {
                                // VBlank
                                self.registers_mut().pc.value = 0x40;
                            }
                            0b0000_0010 => {
                                // LCDStat
                                self.registers_mut().pc.value = 0x48;
                            }
                            0b0000_0100 => {
                                // TimerOverflow
                                self.registers_mut().pc.value = 0x50;
                            }
                            0b0000_1000 => {
                                // SerialTranferComplete
                                self.registers_mut().pc.value = 0x58;
                            }
                            0b0001_0000 => {
                                // Joypad
                                self.registers_mut().pc.value = 0x60;
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
    }

    fn interrupt(&mut self, interrupt: Interrupt) {
        let interrupt_flag = self.read_u8(locations::IF);
        match interrupt {
            Interrupt::VBlank => {
                self.write_u8(locations::IF, interrupt_flag | 0b0000_0001);
            }
            Interrupt::LCDStat => {
                self.write_u8(locations::IF, interrupt_flag | 0b0000_0010);
            }
            Interrupt::TimerOverflow => {
                self.write_u8(locations::IF, interrupt_flag | 0b0000_0100);
            }
            Interrupt::SerialTranferComplete => {
                self.write_u8(locations::IF, interrupt_flag | 0b0000_1000);
            }
            Interrupt::Joypad => {
                self.write_u8(locations::IF, interrupt_flag | 0b0001_0000);
            }
        }
    }

    /// TODO: CHANGE VALUES WHEN IMPLEMENTING THE GAMEBOY COLOR (CGB)
    /// TODO: [REFERENCE](https://gbdev.io/pandocs/Power_Up_Sequence.html)
    fn reset(&mut self) {
        self.memory_mut().fill_with(rand::random);
        self.ram_mut().fill_with(rand::random);

        self.registers_mut().af.hi = 0x01; // TODO: 0x11 if GBColor
        self.registers_mut().af.lo = if self.memory()[locations::COMPLEMENT_CHECK] == 0x00 {
            0b1000_0000
        } else {
            0b1011_0000
        };
        self.registers_mut().bc.lo = 0x13;
        self.registers_mut().de.lo = 0xD8;
        self.registers_mut().hl.hi = 0x01;
        self.registers_mut().hl.lo = 0x4D;
        self.registers_mut().pc.value = 0x0100;
        self.registers_mut().sp.value = 0xFFFE;
        self.registers_mut().ime = false;

        self.memory_mut()[locations::P1] = 0xCF;
        self.memory_mut()[locations::SB] = 0x00;
        self.memory_mut()[locations::SC] = 0x7E;
        self.memory_mut()[locations::DIV] = 0xAB;
        self.memory_mut()[locations::TIMA] = 0x00;
        self.memory_mut()[locations::TMA] = 0x00;
        self.memory_mut()[locations::TAC] = 0xF8;
        self.memory_mut()[locations::IF] = 0xE1;
        self.memory_mut()[locations::NR10] = 0x80;
        self.memory_mut()[locations::NR11] = 0xBF;
        self.memory_mut()[locations::NR12] = 0xF3;
        self.memory_mut()[locations::NR13] = 0xFF;
        self.memory_mut()[locations::NR14] = 0xBF;
        self.memory_mut()[locations::NR21] = 0x3F;
        self.memory_mut()[locations::NR22] = 0x00;
        self.memory_mut()[locations::NR23] = 0xFF;
        self.memory_mut()[locations::NR24] = 0xBF;
        self.memory_mut()[locations::NR30] = 0x7F;
        self.memory_mut()[locations::NR31] = 0xFF;
        self.memory_mut()[locations::NR32] = 0x9F;
        self.memory_mut()[locations::NR33] = 0xFF;
        self.memory_mut()[locations::NR34] = 0xBF;
        self.memory_mut()[locations::NR41] = 0xFF;
        self.memory_mut()[locations::NR42] = 0x00;
        self.memory_mut()[locations::NR43] = 0x00;
        self.memory_mut()[locations::NR44] = 0xBF;
        self.memory_mut()[locations::NR50] = 0x77;
        self.memory_mut()[locations::NR51] = 0xF3;
        self.memory_mut()[locations::NR52] = 0xF1; // TODO: 0xF0 if SGB
        self.memory_mut()[locations::LCDC] = 0x91;
        self.memory_mut()[locations::STAT] = 0x85;
        self.memory_mut()[locations::SCY] = 0x00;
        self.memory_mut()[locations::SCX] = 0x00;
        self.memory_mut()[locations::LY] = 0x00;
        self.memory_mut()[locations::LYC] = 0x00;
        self.memory_mut()[locations::DMA] = 0xFF;
        self.memory_mut()[locations::BGP] = 0xFC;
        self.memory_mut()[locations::OBP0] = 0xFF;
        self.memory_mut()[locations::OBP1] = 0xFF;
        self.memory_mut()[locations::WY] = 0x00;
        self.memory_mut()[locations::WX] = 0x00;
        self.memory_mut()[locations::IE] = 0x00;
    }
}

impl Cpu for crate::GameBoy {}
