use std::ops::RangeInclusive;

/// Usually a NOP instruction and a JP
pub const ENTRYPOINT: RangeInclusive<usize> = 0x0100..=0x0103;
/// Must be exact for the cartridge to be executed
pub const NINTENDO_GRAPHICS: RangeInclusive<usize> = 0x0104..=0x0133;
/// Title of the game in UPPERCASE ASCII
pub const GAME_TITLE_OLDER: RangeInclusive<usize> = 0x0134..=0x0143;
/// Title of the game in UPPERCASE ASCII
pub const GAME_TITLE: RangeInclusive<usize> = 0x0134..=0x013E;
/// Manufacturer code
pub const MANUFACTURER_CODE: RangeInclusive<usize> = 0x013F..=0x0142;
/// Indicates if the game is Color GB
pub const COLOR_INDICATOR: usize = 0x0143;
/// New Licensee Code (when old code is 0x33)
pub const LICENSEE_CODE: RangeInclusive<usize> = 0x0144..=0x145;
/// Indicates if the game is SGB or GB
pub const GB_SGB_INDICATOR: usize = 0x0146;
/// Cartridge type
pub const CARTRIDGE_TYPE: usize = 0x0147;
/// Rom size in banks
pub const ROM_SIZE: usize = 0x0148;
/// Ram size in banks
pub const RAM_SIZE: usize = 0x0149;
/// Destination code
pub const DESTINATION_CODE: usize = 0x014A;
pub const LICENSEE_CODE_OLDER: usize = 0x014B;
pub const MASK_ROM_VERSION_NUMBER: usize = 0x014C;
pub const COMPLEMENT_CHECK: usize = 0x014D;
pub const CHECKSUM: RangeInclusive<usize> = 0x014E..=0x014F;

/// Register for reading joy pad info and determining system type.
pub const P1: usize = 0xFF00;
/// Serial transfer data
pub const SB: usize = 0xFF01;
/// Serial transfer control
pub const SC: usize = 0xFF02;

/// Divider Register
///
/// This register is incremented 16384 (~16779 on SGB) times a second.
/// Writing any value sets it to $00.
pub const DIV: usize = 0xFF04;

/// Timer counter
///
/// This timer is incremented by a clock frequency specified by the TAC register ($FF07).
/// The timer generates an interrupt when it overflows.
pub const TIMA: usize = 0xFF05;
/// Timer Modulo
///
/// When the TIMA overflows, this data will be loaded.
pub const TMA: usize = 0xFF06;
/// Timer control
///
/// - Bit 2 - Timer Stop
///   - 0: Stop Timer
///   - 1: Start Timer
/// - Bits 1+0 - Input Clock Select
///   - 00: 4.096 KHz (~4.194 KHz SGB)
///   - 01: 262.144 Khz (~268.4 KHz SGB)
///   - 10: 65.536 KHz (~67.11 KHz SGB)
///   - 11: 16.384 KHz (~16.78 KHz SGB)
pub const TAC: usize = 0xFF07;

/// Interrupt Flag
///
/// - Bit 4: Transition from High to Low of Pin number P10-P13
/// - Bit 3: Serial I/O transfer complete
/// - Bit 2: Timer Overflow
/// - Bit 1: LCDC (see STAT)
/// - Bit 0: V-Blank
pub const IF: usize = 0xFF0F;

/// Sound mode 1 register, sweep register
pub const NR10: usize = 0xFF10;
/// Sound Mode 1 register, Sound length/Wave pattern duty
pub const NR11: usize = 0xFF11;
/// Sound Mode 1 register, Envelope
pub const NR12: usize = 0xFF12;
/// Sound Mode 1 register, frequency lo
pub const NR13: usize = 0xFF13;
/// Sound Mode 1 register, frequency hi
pub const NR14: usize = 0xFF14;

/// Sound Mode 2 register, sound length/wave pattern duty
pub const NR21: usize = 0xFF16;
/// Sound Mode 2 register, envelope
pub const NR22: usize = 0xFF17;
/// Sound Mode 2 register, frequency lo
pub const NR23: usize = 0xFF18;
/// Sound Mode 2 register, frequency hi
pub const NR24: usize = 0xFF19;

/// Sound Mode 3 register, sound on/off
pub const NR30: usize = 0xFF1A;
/// Sound Mode 3 register, sound length
pub const NR31: usize = 0xFF1A;
/// Sound Mode 3 register, select output level
pub const NR32: usize = 0xFF1C;
/// Sound Mode 3 register, frequency lo
pub const NR33: usize = 0xFF1D;
/// Sound Mode 3 register, frequency hi
pub const NR34: usize = 0xFF1E;

/// Sound Mode 4 register, sound length
pub const NR41: usize = 0xFF20;
/// Sound Mode 4 register, envelope
pub const NR42: usize = 0xFF21;
/// Sound Mode 4 register, polynomial counter
pub const NR43: usize = 0xFF22;
/// Sound Mode 4 register, counter/consecutive; initial
pub const NR44: usize = 0xFF23;

/// Channel control / ON-OFF / Volume
pub const NR50: usize = 0xFF24;
/// Selection of Sound output terminal
pub const NR51: usize = 0xFF25;
/// Sound on/off
pub const NR52: usize = 0xFF26;

/// Waveform storage for arbitrary sound data
pub const WAVE_PATTERN_RAM: RangeInclusive<usize> = 0xFF30..=0xFF3F;

/// LCD control
///
///
/// - Bit 7 - LCD Control Operation *
///   - 0: Stop completely (no picture on screen)
///   - 1: operation
/// - Bit 6 - Window Tile Map Display Select
///   - 0: $9800-$9BFF
///   - 1: $9C00-$9FFF
/// - Bit 5 - Window Display
///   - 0: off
///   - 1: on
/// - Bit 4 - BG & Window Tile Data Select
///   - 0: $8800-$97FF
///   - 1: $8000-$8FFF <- Same area as OBJ
/// - Bit 3 - BG Tile Map Display Select
///   - 0: $9800-$9BFF
///   - 1: $9C00-$9FFF
/// - Bit 2 - OBJ (Sprite) Size
///   - 0: 8*8
///   - 1: 8\*16 (width\*height)
/// - Bit 1 - OBJ (Sprite) Display
///   - 0: off
///   - 1: on
/// - Bit 0 - BG & Window Display
///   - 0: off
///   - 1: on
pub const LCDC: usize = 0xFF40;
/// LCDC Status
pub const STAT: usize = 0xFF41;
/// Scroll Y
pub const SCY: usize = 0xFF42;
/// Scroll X
pub const SCX: usize = 0xFF43;
/// LCDC Y-Coordinate
pub const LY: usize = 0xFF44;
/// LY Compare
pub const LYC: usize = 0xFF45;

/// DMA Transfer and Start Address
pub const DMA: usize = 0xFF46;

/// BG & Window Palette Data
///
/// - Bit 7-6 - Data for Dot Data 11 (Normally darkest color)
/// - Bit 5-4 - Data for Dot Data 10
/// - Bit 3-2 - Data for Dot Data 01
/// - Bit 1-0 - Data for Dot Data 00 (Normally lightest color)
///
/// This selects the shade of grays to use for the background (BG) & window pixels.
/// Since each pixel uses 2 bits, the corresponding shade will be selected from here.
pub const BGP: usize = 0xFF47;
/// Object Palette 0 Data
///
/// This selects the colors for sprite palette 0. It works exactly as BGP ($FF47) except each each value of 0 is transparent.
pub const OBP0: usize = 0xFF48;
/// Object Palette 1 Data
///
/// This Selects the colors for sprite palette 1. It works exactly as OBP0 ($FF48). See BGP for details.
pub const OBP1: usize = 0xFF49;

/// Window Y Position
///
/// 0 <= WY <= 143
pub const WY: usize = 0xFF4A;
/// Window X Position
///
/// 0 <= WX <= 166
pub const WX: usize = 0xFF4B;

/// Interrupt Enable
///
/// - Bit 4: Transition from High to Low of Pin number P10-P13.
/// - Bit 3: Serial I/O transfer complete
/// - Bit 2: Timer Overflow
/// - Bit 1: LCDC (see STAT)
/// - Bit 0: V-Blank
///
/// Values
/// - 0: disable
/// - 1: enable
pub const IE: usize = 0xFF40;
