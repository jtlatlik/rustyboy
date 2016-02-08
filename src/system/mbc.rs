use rom::*;
use rom::header::CartridgeType as CType;
use self::MBCType::*;
use super::system::MemoryAccess;
use std::cmp::min;

const EXT_RAM_BANK_SIZE :usize = 8*1024;

type ExtRAMBank = Box<[u8; EXT_RAM_BANK_SIZE]>;

enum MBCType {
	None,
	MBC1,
	MBC2,
	MBC3,
	MBC4,
	MBC5,
	HuC1
}


pub struct MBC {
	rom : Rom,
	ram : ExtRAMBank,
	ctype : MBCType,
	rom_bank : u8,
	ram_bank : u8,
	ram_mode : bool
}

impl MBC {
	
	pub fn new(rom : Rom) -> MBC {
		
		let ctype = match rom.rom_type {
			CType::ROM_ONLY |CType::ROM_RAM | CType::ROM_RAM_BATTERY => None,
			CType::MBC1 | CType::MBC1_RAM | CType::MBC1_RAM_BATTERY => MBC1,
			CType::MBC2 | CType::MBC2_BATTERY => MBC2,
			CType::MBC3 | CType::MBC3_RAM | CType::MBC3_RAM_BATTERY | CType::MBC3_TIMER_BATTERY | CType::MBC3_TIMER_RAM_BATTERY => MBC3,
			CType::MBC4 | CType::MBC4_RAM | CType::MBC4_RAM_BATTERY => MBC4,
			CType::MBC5 | CType::MBC5_RAM | CType::MBC5_RAM_BATTERY | CType::MBC5_RUMBLE | CType::MBC5_RUMBLE_RAM | CType::MBC5_RUMBLE_RAM_BATTERY => MBC5,
			CType::HUC1_RAM_BATTERY => HuC1,
			_ => unimplemented!()
		};
		
		MBC {
			rom : rom,
			ram : Box::new([0; EXT_RAM_BANK_SIZE]),
			ctype : ctype,
			rom_bank : 1,
			ram_bank : 0,
			ram_mode : false
		}
	}
}

impl MemoryAccess for MBC {
	
	fn read(&mut self, addr: u16) -> u8 {
		
		match addr {
			0x0000 ... 0x3fff => self.rom.banks[0][addr as usize],
			0x4000 ... 0x7fff => self.rom.banks[self.rom_bank as usize][(addr - 0x4000) as usize],
			0xa000 ... 0xbfff => self.ram[(addr - 0xa000) as usize],
			_ => unimplemented!()
		}
	}
	
	fn write(&mut self, addr: u16, data: u8) {
		match addr {
			0xa000 ... 0xbfff => self.ram[(addr - 0xa000) as usize] = data,
			_ => {
				match self.ctype {
					None => (), //ignore writes if no MBC present
					MBC1 => match addr {
						0x0000 ... 0x1fff => (), //RAM disable/enable
						0x2000 ... 0x3fff => self.rom_bank = (self.rom_bank & 0xe0) | min(1, data & 0x1f),
						0x4000 ... 0x5fff => {
							if self.ram_mode {
								self.ram_bank = data& 0x3;
							} else {
								self.rom_bank &= (self.rom_bank & 0x1f) | (data & 0x3)<<5;
							}
						},
						0x6000 ... 0x7fff => self.ram_mode = data != 0,
						_ => unreachable!()
					},
					_ => unimplemented!()
				}
			}
		}

	}
}