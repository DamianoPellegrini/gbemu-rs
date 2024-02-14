//! # GameBoy Emulator
//!
//! This is a GameBoy emulator written in Rust.
//!
//! This project is based on information found on the [GameBoy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
//! and the [Pan Docs](https://gbdev.io/pandocs/About.html).
use cartridge::{CartridgeHeader, CartridgeHolder};
use cpu::{Cpu, RegisterFile, Registers};
use instructions::InstructionDecoder;
use memory::{Memory, MemoryMode, Read, Write};

pub mod cartridge;
pub mod cpu;
pub mod instructions;
pub mod memory;
pub mod timer;

pub(crate) const ROM_BANK_SIZE: usize = 0x4000;
pub(crate) const RAM_BANK_SIZE: usize = 0x2000;
pub(crate) const MAX_ROM_BANKS: usize = 0x80;
pub(crate) const MAX_RAM_BANKS: usize = 0x10;

pub struct GameBoy {
    cartridge_header: CartridgeHeader,
    memory_mode: MemoryMode,
    registers: cpu::RegisterFile,
    /// ### Gameboy memory (RAM)
    memory: [u8; 0x10000],
    /// ### Cartridge memory (ROM Banks)
    /// We load all the cartridge in memory without swapping,
    /// only dinamically change addressing
    cartridge: Vec<u8>,
    /// ### RAM Banks
    /// We keep all banks loaded in memory without swapping,
    /// only dinamically change addressing
    banks: Vec<u8>,
}

impl GameBoy {
    pub fn new(cartridge: &[u8]) -> Self {
        let ch = CartridgeHeader::from(cartridge);

        if (ch.ram_size as usize) > MAX_RAM_BANKS {
            panic!("RAM size is too big");
        }

        if (ch.rom_size as usize) > MAX_ROM_BANKS {
            panic!("ROM size is too big");
        }

        let mut cart = vec![0; ROM_BANK_SIZE * ch.rom_size as usize];
        cart.copy_from_slice(cartridge);

        let mut tmp = Self {
            registers: cpu::RegisterFile::default(),
            memory: [0; 0x10000],
            memory_mode: ch.cart_type.into(),
            cartridge: cart,
            banks: vec![0; RAM_BANK_SIZE * ch.ram_size as usize],
            cartridge_header: ch,
        };

        tmp.reset();

        tmp
    }
}

impl Memory for GameBoy {
    fn cartridge(&self) -> &[u8] {
        &self.cartridge
    }

    fn cartridge_mut(&mut self) -> &mut [u8] {
        &mut self.cartridge
    }

    fn ram(&self) -> &[u8] {
        &self.banks
    }

    fn ram_mut(&mut self) -> &mut [u8] {
        &mut self.banks
    }

    fn memory(&self) -> &[u8; 0x10000] {
        &self.memory
    }

    fn memory_mut(&mut self) -> &mut [u8; 0x10000] {
        &mut self.memory
    }

    fn memory_mode(&self) -> MemoryMode {
        self.memory_mode
    }

    fn memory_mode_mut(&mut self) -> &mut MemoryMode {
        &mut self.memory_mode
    }
}

impl Read for GameBoy {}
impl Write for GameBoy {}

impl Registers for GameBoy {
    fn registers(&self) -> &RegisterFile {
        &self.registers
    }

    fn registers_mut(&mut self) -> &mut RegisterFile {
        &mut self.registers
    }
}

impl InstructionDecoder for GameBoy {}

impl CartridgeHolder for GameBoy {
    fn cartridge_header(&self) -> CartridgeHeader {
        self.cartridge_header.clone()
    }
}
