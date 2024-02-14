use crate::memory::{locations, Memory};

#[derive(Debug, Clone, Copy)]
pub enum Destination {
    Japanese = 0x00,
    NonJapanese = 0x01,
}

impl From<u8> for Destination {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Japanese,
            0x01 => Self::NonJapanese,
            _ => panic!("Invalid destination value"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Indicates the mapper present on the cartridge
pub enum CartridgeType {
    RomOnly,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    NotSupported,
    Unknown,
}

impl From<u8> for CartridgeType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::RomOnly,
            0x01..=0x03 => Self::MBC1,
            0x05..=0x06 => Self::MBC2,
            0x0F..=0x13 => Self::MBC3,
            0x19..=0x1E => Self::MBC5,
            0x08..=0x09 | 0x20 | 0x22 | 0xFC..=0xFF => Self::NotSupported,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Rom size in banks
pub enum RomSize {
    /// No banking
    KiB32 = 0x02,
    KiB64 = 0x04,
    KiB128 = 0x08,
    KiB256 = 0x10,
    KiB512 = 0x20,
    MiB1 = 0x40,
    MiB2 = 0x80,
    MiB4 = 0x100,
    MiB8 = 0x200,
    MiB1Point1 = 0x48,
    MiB1Point2 = 0x50,
    MiB1Point5 = 0x60,
}

impl From<u8> for RomSize {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::KiB32,
            0x01 => Self::KiB64,
            0x02 => Self::KiB128,
            0x03 => Self::KiB256,
            0x04 => Self::KiB512,
            0x05 => Self::MiB1,
            0x06 => Self::MiB2,
            0x07 => Self::MiB4,
            0x08 => Self::MiB8,
            0x52 => Self::MiB1Point1,
            0x53 => Self::MiB1Point2,
            0x54 => Self::MiB1Point5,
            _ => panic!("Invalid rom size value"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Ram size in banks
pub enum RamSize {
    None = 0x00,
    KiB8 = 0x01,
    KiB32 = 0x04,
    KiB128 = 0x10,
    KiB64 = 0x08,
}

impl From<u8> for RamSize {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::None,
            0x02 => Self::KiB8,
            0x03 => Self::KiB32,
            0x04 => Self::KiB128,
            0x05 => Self::KiB64,
            _ => panic!("Invalid ram size value"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CartridgeHeader {
    /// Title of the game in uppercase ASCII
    pub title: String,
    /// Game supports Color-Mode
    pub color: bool,
    /// Game supports SGB functions
    pub sgb: bool,
    /// Indicates what kind of hardware is present on the cartridge (notably the mapper).
    pub cart_type: CartridgeType,
    /// How much ROM is present on the cartridge.
    pub rom_size: RomSize,
    /// How much RAM is present on the cartridge.
    pub ram_size: RamSize,
    /// Indicates whether this version of the fame is intended to be sold in Japan or elsewhere.
    pub destination: Destination,
    pub version: u8,
    pub header_checksum: u8,
    pub global_checksum: u16,
}

impl From<&[u8]> for CartridgeHeader {
    fn from(value: &[u8]) -> Self {
        let is_newer = value[locations::LICENSEE_CODE_OLDER] == 0x33;

        CartridgeHeader {
            title: String::from_utf8(
                value[if is_newer {
                    locations::GAME_TITLE
                } else {
                    locations::GAME_TITLE_OLDER
                }]
                .to_vec(),
            )
            .unwrap_or(String::from("Unknown"))
            .trim()
            .to_string(),
            color: value[locations::COLOR_INDICATOR] == 0x80,
            sgb: is_newer && value[locations::GB_SGB_INDICATOR] == 0x03,
            cart_type: CartridgeType::from(value[locations::CARTRIDGE_TYPE]),
            rom_size: RomSize::from(value[locations::ROM_SIZE]),
            ram_size: RamSize::from(value[locations::RAM_SIZE]),
            destination: Destination::from(value[locations::DESTINATION_CODE]),
            version: value[locations::MASK_ROM_VERSION_NUMBER],
            header_checksum: value[locations::COMPLEMENT_CHECK],
            global_checksum: u16::from_be_bytes(value[locations::CHECKSUM].try_into().unwrap()),
        }
    }
}

pub trait CartridgeHolder: Memory {
    fn cartridge_header(&self) -> CartridgeHeader {
        CartridgeHeader::from(self.cartridge())
    }
}
