use crate::{cartridge::CartridgeType, RAM_BANK_SIZE};

pub mod locations;

#[derive(Debug, Clone, Copy)]
/// Indicates how the controller should behave
pub enum MemoryMode {
    RomOnly,
    MBC1 {
        rom_bank_idx: usize,
        ram_bank_idx: usize,
        ram_enabled: bool,
        /// If true address 0x4000..=0x5FFF selects ram bank,
        /// select upper bits of ROM bank otherwise
        ram_banking: bool,
    },
    MBC2 {
        rom_bank_idx: usize,
        ram_enabled: bool,
    },
    MBC3 {
        rom_bank_idx: usize,
        ram_bank_idx: usize,
        ram_rtc_enabled: bool,
        /// If true address 0xA000..=0xBFFF points to RTC registers,
        /// points to ram bank otherwise
        rtc_selected: Option<u8>,
        /// If true RTC registers are latched (don't update)
        rtc_latched: bool,
        /// Seconds register for RTC
        rtc_seconds: u8,
        /// Minutes register for RTC
        rtc_minutes: u8,
        /// Hours register for RTC
        rtc_hours: u8,
        /// Days register for RTC
        ///
        /// - Bit 0: MSB of day counter
        /// - Bit 6: Halt RTC (0 = Active, 1 = Halt)
        /// - Bit 7: Day counter carry bit (1 = Counter overflow)
        rtc_days: u16,
    },
    MBC5 {
        rom_bank_idx: usize,
        ram_bank_idx: usize,
        ram_enabled: bool,
        rumble_enabled: bool,
    },
}

impl From<CartridgeType> for MemoryMode {
    fn from(value: CartridgeType) -> Self {
        match value {
            CartridgeType::RomOnly => Self::RomOnly,
            CartridgeType::MBC1 => Self::MBC1 {
                rom_bank_idx: 1,
                ram_bank_idx: 0,
                ram_enabled: false,
                ram_banking: true,
            },
            CartridgeType::MBC2 => Self::MBC2 {
                rom_bank_idx: 1,
                ram_enabled: false,
            },
            CartridgeType::MBC3 => Self::MBC3 {
                rom_bank_idx: 1,
                ram_bank_idx: 0,
                ram_rtc_enabled: false,
                rtc_selected: None,
                rtc_latched: false,
                rtc_seconds: 0,
                rtc_minutes: 0,
                rtc_hours: 0,
                rtc_days: 0,
            },
            CartridgeType::MBC5 => Self::MBC5 {
                rom_bank_idx: 1,
                ram_bank_idx: 0,
                ram_enabled: false,
                rumble_enabled: false,
            },
            CartridgeType::NotSupported | CartridgeType::Unknown => {
                panic!("Unsupported cartridge type")
            }
        }
    }
}

pub trait Memory {
    /// Returns a slice of the entire memory (0x0000..0xFFFF)
    fn memory(&self) -> &[u8; 0x10000];
    /// Returns a mutable slice of the entire memory (0x0000..0xFFFF)
    fn memory_mut(&mut self) -> &mut [u8; 0x10000];

    /// Returns a slice of the cartridge
    fn cartridge(&self) -> &[u8];
    /// Returns a mutable slice of the cartridge
    fn cartridge_mut(&mut self) -> &mut [u8];

    /// Returns a slice of the RAM
    fn ram(&self) -> &[u8];
    /// Returns a mutable slice of the RAM
    fn ram_mut(&mut self) -> &mut [u8];

    /// Returns the current ROM bank
    fn rom_bank_idx(&self) -> usize {
        match self.memory_mode() {
            MemoryMode::RomOnly => 1,
            MemoryMode::MBC1 { rom_bank_idx, .. } => rom_bank_idx,
            MemoryMode::MBC2 { rom_bank_idx, .. } => rom_bank_idx,
            MemoryMode::MBC3 { rom_bank_idx, .. } => rom_bank_idx,
            MemoryMode::MBC5 { rom_bank_idx, .. } => rom_bank_idx,
        }
    }
    /// Returns the current RAM bank
    fn ram_bank_idx(&self) -> usize {
        match self.memory_mode() {
            MemoryMode::RomOnly => 0,
            MemoryMode::MBC1 { ram_bank_idx, .. } => ram_bank_idx,
            MemoryMode::MBC2 { .. } => 0,
            MemoryMode::MBC3 { ram_bank_idx, .. } => ram_bank_idx,
            MemoryMode::MBC5 { ram_bank_idx, .. } => ram_bank_idx,
        }
    }

    fn memory_mode(&self) -> MemoryMode;
    fn memory_mode_mut(&mut self) -> &mut MemoryMode;
}

pub trait Read: Memory {
    fn read_u8(&self, address: usize) -> u8 {
        match address {
            // Read from ROM Bank 0
            0x0000..=0x3FFF => self.cartridge()[address],
            // Read from ROM Bank
            0x4000..=0x7FFF => {
                self.cartridge()[address - 0x4000 + (self.rom_bank_idx() * crate::ROM_BANK_SIZE)]
            }
            // Read from RAM Bank
            0xA000..=0xBFFF => match self.memory_mode() {
                MemoryMode::MBC1 {
                    ram_bank_idx,
                    ram_enabled,
                    ..
                }
                | MemoryMode::MBC5 {
                    ram_bank_idx,
                    ram_enabled,
                    ..
                } => {
                    if ram_enabled {
                        self.ram()[address - 0xA000 + (ram_bank_idx * crate::RAM_BANK_SIZE)]
                    } else {
                        0
                    }
                }
                MemoryMode::MBC2 { ram_enabled, .. } => {
                    if ram_enabled {
                        let address = address - 0xA000;
                        let address = match address {
                            0xA000..=0xA1FF => address,
                            0xA200..=0xBFFF => address & 0x1FF,
                            _ => unreachable!(),
                        };
                        self.ram()[address]
                    } else {
                        0
                    }
                }
                MemoryMode::MBC3 {
                    ram_bank_idx,
                    ram_rtc_enabled,
                    rtc_selected,
                    rtc_seconds,
                    rtc_minutes,
                    rtc_hours,
                    rtc_days,
                    ..
                } => {
                    if ram_rtc_enabled {
                        if let Some(selected) = rtc_selected {
                            match selected {
                                0x08 => rtc_seconds,
                                0x09 => rtc_minutes,
                                0x0A => rtc_hours,
                                0x0B => (rtc_days & 0xFF) as u8,
                                0x0C => (rtc_days >> 8) as u8,
                                _ => unreachable!(),
                            }
                        } else {
                            self.ram()[address - 0xA000 + (ram_bank_idx * crate::RAM_BANK_SIZE)]
                        }
                    } else {
                        0
                    }
                }
                _ => self.ram()[address - 0xA000 + (self.ram_bank_idx() * crate::RAM_BANK_SIZE)],
            },
            // Echo RAM
            0xE000..=0xFDFF => self.memory()[address - 0x2000],
            _ => self.memory()[address],
        }
    }

    fn read_u16(&self, address: usize) -> u16 {
        let lower = self.read_u8(address) as u16;
        let upper = self.read_u8(address + 1) as u16;
        (upper << 8) | lower
    }

    fn read_bytes(&self, addresses: std::ops::RangeInclusive<usize>) -> Vec<u8> {
        addresses.map(|address| self.read_u8(address)).collect()
    }
}

pub trait Write: Memory {
    fn write_u8(&mut self, address: usize, value: u8) {
        // Handle MBC Registers
        match self.memory_mode_mut() {
            MemoryMode::RomOnly => (),
            MemoryMode::MBC1 {
                rom_bank_idx,
                ram_bank_idx,
                ram_enabled,
                ram_banking,
            } => match address {
                // Ram enable
                0x0000..=0x1FFF => *ram_enabled = value & 0b1111 == 0b1010,
                // Rom bank select
                0x2000..=0x3FFF => {
                    let bank = value & 0b11111;
                    *rom_bank_idx = if bank == 0 { 1 } else { bank as usize };
                }
                // Ram bank select or upper bits of rom bank select
                0x4000..=0x5FFF => {
                    let bank = value as usize & 0b11;
                    if *ram_banking {
                        *ram_bank_idx = bank;
                    } else {
                        *rom_bank_idx = (bank << 5) + (*rom_bank_idx & 0b11111);
                    }
                }
                // Rom/Ram banking mode select
                0x6000..=0x7FFF => *ram_banking = value & 0b1 == 0b1,
                _ => (),
            },
            MemoryMode::MBC2 {
                rom_bank_idx,
                ram_enabled,
            } => {
                // Ram enable/Rom bank select
                if let 0x0000..=0x3FFF = address {
                    let bank_switching = value & (0b1 << 7) == 0b1000_0000;
                    if bank_switching {
                        let bank = value & 0b1111;
                        *rom_bank_idx = if bank == 0 { 1 } else { bank as usize };
                    } else {
                        *ram_enabled = value & 0b1111 == 0b1010;
                    }
                }
            }
            MemoryMode::MBC3 {
                rom_bank_idx,
                ram_bank_idx,
                ram_rtc_enabled,
                rtc_selected,
                rtc_latched,
                ..
            } => match address {
                // Ram enable/Rom bank select
                0x0000..=0x1FFF => *ram_rtc_enabled = value & 0b1111 == 0b1010,
                // Rom bank select
                0x2000..=0x3FFF => {
                    let bank = value & 0b1111111;
                    *rom_bank_idx = if bank == 0 { 1 } else { bank as usize };
                }
                // Ram bank select or RTC register select
                0x4000..=0x5FFF => match value {
                    0x00..=0x03 => {
                        let bank = value as usize & 0b11;
                        *ram_bank_idx = bank;
                        *rtc_selected = None
                    }
                    0x08..=0x0C => *rtc_selected = Some(value),
                    _ => (),
                },
                // Latch clock data
                0x6000..=0x7FFF => *rtc_latched = value & 0b1 == 0b1,
                _ => (),
            },
            MemoryMode::MBC5 {
                rom_bank_idx,
                ram_bank_idx,
                ram_enabled,
                rumble_enabled,
            } => match address {
                // Ram enable
                0x0000..=0x1FFF => *ram_enabled = value & 0b1111 == 0b1010,
                // Rom bank select lower 8 bits
                0x2000..=0x2FFF => {
                    let bank = value as usize;
                    *rom_bank_idx = if bank == 0 { 1 } else { bank };
                }
                // Rom bank select upper bit
                0x3000..=0x3FFF => {
                    let bank = value as usize & 0b1;
                    *rom_bank_idx = (bank << 8) + (*rom_bank_idx & 0b11111111);
                }
                // Ram bank select
                0x4000..=0x5FFF => {
                    // TODO: Check if mask is wrong
                    *ram_bank_idx = value as usize & 0b1111;
                    *rumble_enabled = value & 0b100 == 0b100;
                }
                _ => (),
            },
        };

        // Handle RAM bank writes
        if (0xA000..=0xBFFF).contains(&address) {
            match self.memory_mode() {
                MemoryMode::MBC1 {
                    ram_bank_idx,
                    ram_enabled,
                    ..
                }
                | MemoryMode::MBC5 {
                    ram_bank_idx,
                    ram_enabled,
                    ..
                } => {
                    if ram_enabled {
                        self.ram_mut()[address - 0xA000 + ram_bank_idx * RAM_BANK_SIZE] = value;
                    }
                }
                MemoryMode::MBC3 {
                    ram_bank_idx,
                    ram_rtc_enabled,
                    rtc_selected,
                    ..
                } => {
                    if rtc_selected.is_none() && ram_rtc_enabled {
                        self.ram_mut()[address - 0xA000 + ram_bank_idx * RAM_BANK_SIZE] = value;
                    }
                }

                MemoryMode::MBC2 { ram_enabled, .. } => match address {
                    0xA000..=0xA1FF => {
                        if ram_enabled {
                            self.ram_mut()[address - 0xA000] = value;
                        }
                    }
                    0xA200..=0xBFFF => {
                        if ram_enabled {
                            self.ram_mut()[(address - 0xA000) & 0x1FF] = value;
                        }
                    }
                    _ => (),
                },
                _ => (),
            };

            return; // Written to RAM banks ends here
        }

        // Handle normal writes
        match address {
            // No write zones
            0x0000..=0x7FFF /* ROM */ | 0xFEA0..=0xFEFF /* Restricted */ => (),
            // Echo RAM
            0xE000..=0xFDFF => self.memory_mut()[address - 0x2000] = value,
            // Trap DIV | LY writes
            locations::DIV | locations::LY => self.memory_mut()[address] = 0,
            // Trap timer frequency changes
            locations::TAC => {
                let current_freq = self.memory()[locations::TAC] & 0b11;
                let new_freq = value & 0b11;
                if current_freq != new_freq {
                    self.memory_mut()[locations::TIMA] = 0;
                }
            }
            _ => self.memory_mut()[address] = value,
        }
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        let upper = (value >> 8) as u8;
        let lower = value as u8;
        self.write_u8(address, lower);
        self.write_u8(address + 1, upper);
    }

    fn write_bytes(&mut self, addresses: std::ops::RangeInclusive<usize>, values: &[u8]) {
        for (address, value) in addresses.zip(values) {
            self.write_u8(address, *value);
        }
    }
}
