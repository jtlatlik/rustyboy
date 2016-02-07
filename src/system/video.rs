use std::boxed::Box;

const VRAM_BANK_SIZE :usize = 8*1024; //8K per VRAM bank
pub type VRAMBank = Box<[u8; VRAM_BANK_SIZE]>;

const VBLANK_PERIOD : u32 = 4560;
const HBLANK_PERIOD : u32 = 204;
const SCANLINE_OAM_PERIOD : u32 = 80;
const SCANLINE_VRAM_PERIOD : u32 = 172;

pub const LCD_CONTROL_BGMAP :u32 = 3;

#[derive(Default)]
pub struct VideoRegisters {
	pub lcd_ctrl : u8,
	pub lcd_status : u8,
	pub scy : u8,
	pub scx : u8,
	pub ly : u8,
	pub lyc : u8,
	pub wy : u8,
	pub wx : u8,
	pub bgp : u8,
	pub obp0 : u8,
	pub obp1 : u8
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
				
		match self.regs.lcd_status & 0b11 {
			0 => { //H-BLANK
				if self.mode_cycles >= HBLANK_PERIOD  {
					self.mode_cycles -= HBLANK_PERIOD;
					self.regs.ly += 1;
					if self.regs.ly >= 144 {
						self.regs.lcd_status = self.regs.lcd_status & 0xfc | 1;						
					} else {
						self.regs.lcd_status = self.regs.lcd_status & 0xfc | 2;
					}
				}
			},
			1 => { //V-BLANK
				if self.mode_cycles >= HBLANK_PERIOD {
					self.mode_cycles -= HBLANK_PERIOD;
					self.regs.ly = (self.regs.ly + 1) % 154;
				}
				if self.regs.ly == 0  {
					self.regs.lcd_status = self.regs.lcd_status & 0xfc | 2;
				}
			},
			2 => { //SCANLINE OAM
				if self.mode_cycles >= SCANLINE_OAM_PERIOD {
					self.mode_cycles -= SCANLINE_OAM_PERIOD;
					self.regs.lcd_status = self.regs.lcd_status & 0xfc | 3;
				}
			},
			3 => { //SCANLINE VRAM
				if self.mode_cycles >= SCANLINE_VRAM_PERIOD {
					self.mode_cycles -= SCANLINE_VRAM_PERIOD;
					self.regs.lcd_status = self.regs.lcd_status & 0xfc | 0;
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
			regs : VideoRegisters { ..Default::default() },
			oam : [0; 160],
			mode_cycles : 0
		}
	}
}