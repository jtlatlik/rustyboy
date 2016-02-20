use rom::*;
use rom::header::CartridgeType as CType;
use super::system::MemoryAccess;
use std::cmp::max;
use std::fs::{File,OpenOptions};
use std::path::Path;
use std::io::{Read,Write,Seek,SeekFrom};

const EXT_RAM_BANK_SIZE: usize = 8*1024;

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
	pub rom : Rom,
	ram : Box<[u8]>,
	ctype : MBCType,
	rom_bank : u8,
	ram_bank : usize,
	ram_mode : bool,
	ram_enabled : bool,
	save_file : Option<String>
}

impl MBC {
	
	pub fn new(rom : Rom) -> MBC {
		
		let ctype = match rom.rom_type {			
			CType::ROM_ONLY |CType::ROM_RAM | CType::ROM_RAM_BATTERY => MBCType::None,
			CType::MBC1 | CType::MBC1_RAM | CType::MBC1_RAM_BATTERY => MBCType::MBC1,
			CType::MBC2 | CType::MBC2_BATTERY => MBCType::MBC2,
			CType::MBC3 | CType::MBC3_RAM | CType::MBC3_RAM_BATTERY | CType::MBC3_TIMER_BATTERY | CType::MBC3_TIMER_RAM_BATTERY => MBCType::MBC3,
			CType::MBC4 | CType::MBC4_RAM | CType::MBC4_RAM_BATTERY => MBCType::MBC4,
			CType::MBC5 | CType::MBC5_RAM | CType::MBC5_RAM_BATTERY | CType::MBC5_RUMBLE | CType::MBC5_RUMBLE_RAM | CType::MBC5_RUMBLE_RAM_BATTERY => MBCType::MBC5,
			CType::HUC1_RAM_BATTERY => MBCType::HuC1,
			_ => unimplemented!()
		};

		let mut ram = vec![0; rom.ram_size.as_usize()].into_boxed_slice();
		let mut save_file : Option<String> = None;
		if rom.has_battery() {
			let path_wo_extension = rom.filename.rsplitn(2, '.').last().unwrap();
			let filename = path_wo_extension.to_string() + ".sav";
			save_file = Some(filename.clone());
			let save_filepath = Path::new(&filename); 
			
			//try to load save file
			if save_filepath.exists() {
				println!("found savegame file");
				match File::open(save_filepath) {
					Ok(mut f) => {
					f.read(&mut ram).unwrap_or(0);
				},
					Err(e) => println!("Couldn't open savegame file: {}", e)
				}
			}
		}
		
		MBC {
			rom : rom,
			ram : ram,
			ctype : ctype,
			rom_bank : 1,
			ram_bank : 0,
			ram_mode : false,
			ram_enabled : false,
			save_file : save_file
		}
	}
	
	#[inline(always)]
	pub fn read(&mut self, addr: u16) -> u8 {
		
		match addr >> 8 {
			0x00 ... 0x3f => self.rom.banks[0][addr as usize],
			0x40 ... 0x7f => self.rom.banks[self.rom_bank as usize][(addr - 0x4000) as usize],
			0xa0 ... 0xbf => if self.ram_enabled { self.ram[(self.ram_bank*EXT_RAM_BANK_SIZE + (addr - 0xa000) as usize)] } else { 0xff },
			_ => unimplemented!()
		}
	}
	
	pub fn write(&mut self, addr: u16, data: u8) {
		use self::MBCType::*;
		match addr >> 8 {
			0xa0 ... 0xbf => if self.ram_enabled {
				let ix = self.ram_bank*EXT_RAM_BANK_SIZE + (addr - 0xa000) as usize;
				self.ram[ix] = data;
				if self.rom.has_battery() {
					if let Some(ref save_file) = self.save_file {
						let path = Path::new(save_file);
						if path.exists() {
							//update ram cell in save file
							match OpenOptions::new().write(true).open(path) {
								Ok(mut f) => {
									f.seek(SeekFrom::Start(ix as u64)).unwrap();
									match f.write(&[data]) {
										Ok(_) => (),
										Err(e) => println!("Couldn't open savegame file: {}", e)
									}
									f.flush().unwrap();
							},
								Err(e) => println!("Couldn't open savegame file: {}", e)
							}
						} else {
							//create save file and store ram in it
							match File::create(path) {
								Ok(mut f) => f.write_all(&mut self.ram).unwrap(),
								Err(e) => println!("Couldn't create savegame file: {}", e)
							}
						}
					}
					
				}
			},
			_ => {
				match self.ctype {
					None => (), //ignore writes if no MBC present
					MBC1 => match addr >> 8 {
						0x00 ... 0x1f => { //RAM disable/enable
							self.ram_enabled = (data & 0xf) == 0xa 
						}, 
						0x20 ... 0x3f => self.rom_bank = (self.rom_bank & 0xe0) | max(1, data & 0x1f),
						0x40 ... 0x5f => {
							if self.ram_mode {
								self.ram_bank = (data & 0x3) as usize;
							} else {
								self.rom_bank &= (self.rom_bank & 0x1f) | ((data & 0x3)<<5);
							}
						},
						0x60 ... 0x7f => self.ram_mode = data != 0,
						_ => unreachable!()
					},
					_ => unimplemented!()
				}
			}
		}

	}
}