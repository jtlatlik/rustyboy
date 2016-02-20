use self::RamSize::*;

#[derive(Copy, Clone,Debug)]
pub enum CartridgeType {
    ROM_ONLY = 0x00,
    MBC1 = 0x01,
    MBC1_RAM =0x02,
    MBC1_RAM_BATTERY = 0x03,
    MBC2 = 0x05,
    MBC2_BATTERY = 0x06,
    ROM_RAM = 0x08,
    ROM_RAM_BATTERY = 0x09,
    MMM01 = 0x0b,
    MMM01_RAM = 0x0c,
    MMM01_RAM_BATTERY = 0x0d,
    MBC3_TIMER_BATTERY = 0x0f,
    MBC3_TIMER_RAM_BATTERY = 0x10,
    MBC3 = 0x11,
    MBC3_RAM = 0x12,
    MBC3_RAM_BATTERY = 0x13,
    MBC4 = 0x15,
    MBC4_RAM = 0x16,
    MBC4_RAM_BATTERY = 0x17,
    MBC5 = 0x19,
    MBC5_RAM = 0x1a,
    MBC5_RAM_BATTERY = 0x1b,
    MBC5_RUMBLE = 0x1c,
    MBC5_RUMBLE_RAM = 0x1d,
    MBC5_RUMBLE_RAM_BATTERY = 0x1e,
    POCKET_CAMERA = 0xfc,
    BANDAI_TAMA5 = 0xfd,
    HUC3 = 0xfe,
    HUC1_RAM_BATTERY = 0xff
}

#[derive(Copy, Clone,Debug)]
pub enum RomSize {
    ROM_32K = 0x00,
    ROM_64K = 0x01,
    ROM_128K = 0x02, 
    ROM_256K = 0x03,
    ROM_512K = 0x04,
    ROM_1M = 0x05,
    ROM_2M = 0x06,
    ROM_4M = 0x07,
    ROM_1M1 = 0x52,
    ROM_1M2 = 0x53,
    ROM_1M5 = 0x54
}

#[derive(Copy, Clone,Debug)]
pub enum RamSize {
    RAM_NONE = 0x00,
    RAM_2K = 0x01,
    RAM_8K = 0x02,
    RAM_32K = 0x03,
    RAM_128K = 0x04,
    RAM_64K = 0x05
}

impl RamSize {
	
	pub fn as_usize(self) -> usize {
			match self {
				RAM_NONE => 0,
				RAM_2K => 2048,
				RAM_8K => 8192,
				RAM_32K => 32768,
				RAM_64K => 64*1024,
				RAM_128K => 128*1024,
			}
	}
}


