use std::boxed::Box;
use super::system::MemoryAccess;
use super::ioregister::IORegister;

const VRAM_BANK_SIZE :usize = 8*1024; //8K per VRAM bank

pub type VRAMBank = Box<[u8; VRAM_BANK_SIZE]>;

const VBLANK_PERIOD : u32 = 4560;
const HBLANK_PERIOD : u32 = 204;
const SCANLINE_OAM_PERIOD : u32 = 80;
const SCANLINE_VRAM_PERIOD : u32 = 172;

pub const LCD_CONTROL_BGMAP :u32 = 3;
pub const LCD_CONTROL_TSSEL :u32 = 4;

macro_rules! bits {
	( $($bit:expr)* ) => ( 0x00 $( | (1<<$bit) )* )
}

pub struct LCDControlRegister(pub u8);

#[derive(Default)]
pub struct VideoRegisters {
	pub lcd_ctrl : IORegister,
	pub lcd_status : IORegister,
	pub scy : IORegister,
	pub scx : IORegister,
	pub ly : IORegister,
	pub lyc : IORegister,
	pub wy : IORegister,
	pub wx : IORegister,
	pub bgp : IORegister,
	pub obp0 : IORegister,
	pub obp1 : IORegister
}

pub struct VideoData {
	pub regs : VideoRegisters,
	pub vram0 : VRAMBank,
	pub oam : [u8; 160],
	
	pub mode_cycles : u32
}

impl VideoData {
	
	pub fn update(&mut self, delta : u32) {
		self.mode_cycles += delta;
		
		let status = *self.regs.lcd_status;
				
		match status & 0b11 {
			0 => { //H-BLANK
				if self.mode_cycles >= HBLANK_PERIOD  {
					self.mode_cycles -= HBLANK_PERIOD;
					*self.regs.ly += 1;
					if *self.regs.ly >= 144 {
						*self.regs.lcd_status = status & 0xfc | 1;
						//TODO request VBLANK interrupt						
					} else {
						*self.regs.lcd_status = status & 0xfc | 2;
					}
				}
			},
			1 => { //V-BLANK
				if self.mode_cycles >= HBLANK_PERIOD {
					self.mode_cycles -= HBLANK_PERIOD;
					*self.regs.ly = (*self.regs.ly + 1) % 154;
				}
				if *self.regs.ly == 0  {
					*self.regs.lcd_status = status & 0xfc | 2;
				}
			},
			2 => { //SCANLINE OAM
				if self.mode_cycles >= SCANLINE_OAM_PERIOD {
					self.mode_cycles -= SCANLINE_OAM_PERIOD;
					*self.regs.lcd_status = status & 0xfc | 3;
				}
			},
			3 => { //SCANLINE VRAM
				if self.mode_cycles >= SCANLINE_VRAM_PERIOD {
					self.mode_cycles -= SCANLINE_VRAM_PERIOD;
					*self.regs.lcd_status = status & 0xfc | 0;
					//TODO request HBLANK interrupt
				}
			},
			_ => unreachable!()
		}
		
	}
}

impl Default for VideoData {
	fn default() -> VideoData {
		VideoData {
			vram0 : Box::new([0;VRAM_BANK_SIZE]),
			regs : VideoRegisters { 
				lcd_status: IORegister::new().write_mask(bits!(6 5 4 3)),
				ly : IORegister::new().read_only(),
				..Default::default() },
			oam : [0; 160],
			mode_cycles : 0
		}
	}
}

impl MemoryAccess for VideoData {
	fn read(&mut self, addr: u16) -> u8 {
		match addr {
			0x8000 ... 0x9fff => self.vram0[(addr - 0x8000) as usize],
			0xfe00 ... 0xfe9f => self.oam[(addr - 0xfe00) as usize],
			_ => unimplemented!()
		}
	}
	
	fn write(&mut self, addr: u16, data: u8) {
		match addr {
			0x8000 ... 0x9fff => self.vram0[(addr - 0x8000) as usize] = data,
			0xfe00 ... 0xfe9f => self.oam[(addr - 0xfe00) as usize] = data,
			_ => unimplemented!()
		}
	}
}