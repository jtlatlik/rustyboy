use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::boxed::Box;
use self::VideoMode::*;
use super::interrupt::{self, InterruptRegisters};
use std::cmp::max;
use super::system::MemoryAccess;
use super::ioregister::IORegister;

const VRAM_BANK_SIZE : usize = 8*1024; //8K per VRAM bank
const OAM_NUM_SPRITES : usize = 40;

#[derive(Copy,Clone,PartialEq)]
#[repr(u8)]
pub enum DMGColor {
	White = 0xff,
	LightGray = 0xb6,
	DarkGray = 0x49,
	Black = 0x00
}

type Palette = [DMGColor; 4];
type ColorIndex = u8;

type Tile = [[ColorIndex; 2]; 8]; //2 bytes per row, 8 rows

#[derive(Default,Copy,Clone)]
pub struct Sprite {
	y: u8,
	x : u8,
	tile : u8,
	opt_data : u8,
	priority : bool,
	y_flip : bool,
	x_flip : bool,
	palette_1_sel : bool
}

pub struct VRAMBank {
	
	tile_ram : Box<[Tile; 384]>, 	// 2*(128 unique tiles) + 128 shared tiles
	tile_map: Box<[[u8; 1024]; 2]>, 	// 2 tile maps with 1024 tile indexes each
	accessible : bool
}

pub struct OAM {
	pub sprite_ram : Vec<Sprite>,
	accessible : bool,
	
	pub dma_transfer : bool,
	pub dma_addr : u16,
}


const VBLANK_PERIOD : u32 = 4560;
const HBLANK_PERIOD : u32 = 204;
const SCANLINE_OAM_PERIOD : u32 = 80;
const SCANLINE_VRAM_PERIOD : u32 = 172;

pub const LCD_CONTROL_BGMAP :u8 = 3;
pub const LCD_CONTROL_TSSEL :u8 = 4;
pub const LCD_STATUS_COINCIDENCE : u8 = 2;
pub const LCD_STATUS_HBLANK_INTERRUPT : u8 = 3;
pub const LCD_STATUS_VBLANK_INTERRUPT : u8 = 4;
pub const LCD_STATUS_OAM_INTERRUPT : u8 = 5;
pub const LCD_STATUS_COINCIDENCE_INTERRUPT : u8 = 6;

pub const SCREEN_WIDTH : usize = 160;
pub const SCREEN_HEIGHT : usize = 144;
pub const NUM_SCREEN_PIXELS : usize = SCREEN_WIDTH*SCREEN_HEIGHT;

macro_rules! bits {
	( $($bit:expr)* ) => ( 0x00 $( | (1<<$bit) )* )
}

#[derive(Default)]
pub struct LCDControlRegister {
	data : u8,
	enabled : bool,
	window_tile_map_1_sel : bool,
	window_enabled : bool,
	tile_data_1_sel : bool,
	bg_tile_map_1_sel : bool,
	obj_size_8x16 : bool,
	obj_enabled : bool,
	bg_enabled : bool
}


#[derive(Default)]
pub struct VideoRegisters {
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

#[derive(Copy,Clone)]
enum VideoMode {
	HBLANK = 0,
	VBLANK = 1,
	ACCESS_OAM = 2,
	ACCESS_VRAM = 3
}


pub struct VideoData {
	pub regs : VideoRegisters,
	pub vram0 : VRAMBank,
	pub oam : OAM,
	
	pub lcd_ctrl : LCDControlRegister,
		
	interrupt_regs : Rc<RefCell<InterruptRegisters>>,
	mode : VideoMode,
	mode_cycles : u32,
	bg_palette : Palette,
	obp_palette : [Palette; 2], 
	
	pub back_buffer : Box<[u8; NUM_SCREEN_PIXELS]>,
	pub frame_ready : bool
}

impl VideoData {
	
	pub fn new(iregs : Rc<RefCell<InterruptRegisters>>) -> VideoData {
		use self::DMGColor::*;
		VideoData {
			vram0 : VRAMBank::new(),
			regs : VideoRegisters { 
				lcd_status: IORegister::new().set(2).write_mask(bits!(6 5 4 3)),
				ly : IORegister::new().read_only(),
				..Default::default() },
			lcd_ctrl : LCDControlRegister::new(),
			oam : OAM::new(),
			mode_cycles : 0,
			mode : VBLANK,
			interrupt_regs : iregs,
			bg_palette : [White, LightGray, DarkGray, Black],
			obp_palette : [[White, LightGray, DarkGray, Black]; 2],
			back_buffer : Box::new([DMGColor::LightGray as u8;NUM_SCREEN_PIXELS]),
			frame_ready: false
		}
	}
	
	pub fn update(&mut self, delta : u32) {
		self.mode_cycles += delta;
		//let regs = &mut self.regs;

		match self.mode {
			HBLANK => {
				if self.mode_cycles >= HBLANK_PERIOD  {
					self.mode_cycles -= HBLANK_PERIOD;
					*self.regs.ly += 1;
					self.update_coincidence_flag();
					
					if *self.regs.ly >= 144 {
						self.frame_ready = true;
						self.set_mode(VBLANK);
						{
							let mut iregs = self.interrupt_regs.borrow_mut();
							*iregs.iflags |= interrupt::INTERRUPT_VBLANK
						}
						if *self.regs.lcd_status & (1<<LCD_STATUS_VBLANK_INTERRUPT) != 0 {
							self.request_status_interrupt();							
						}
					} else {
						self.set_mode(ACCESS_OAM);
						if *self.regs.lcd_status & (1<<LCD_STATUS_OAM_INTERRUPT) != 0 {
							self.request_status_interrupt();							
						}
					}
				}
			},
			VBLANK => {
				if self.mode_cycles >= HBLANK_PERIOD {
					self.mode_cycles -= HBLANK_PERIOD;
					*self.regs.ly = (*self.regs.ly + 1) % 154;
					self.update_coincidence_flag();
				}
				if *self.regs.ly == 0  {
					self.set_mode(ACCESS_OAM);
					if *self.regs.lcd_status & (1<<LCD_STATUS_OAM_INTERRUPT) != 0 {
						self.request_status_interrupt();							
					}
				}
			},
			ACCESS_OAM => {
				if self.mode_cycles >= SCANLINE_OAM_PERIOD {
					self.mode_cycles -= SCANLINE_OAM_PERIOD;
					self.set_mode(ACCESS_VRAM);
				}
			},
			ACCESS_VRAM => {
				if self.mode_cycles >= SCANLINE_VRAM_PERIOD {
					self.mode_cycles -= SCANLINE_VRAM_PERIOD;
					self.set_mode(HBLANK);
					self.draw_line();
					if *self.regs.lcd_status & (1<<LCD_STATUS_HBLANK_INTERRUPT) != 0 {
						self.request_status_interrupt();
					}
				}
			}
		}

	}
	
	fn update_coincidence_flag(&mut self) {
		let coincidence = *self.regs.ly == *self.regs.lyc;
		*self.regs.lcd_status = (*self.regs.lcd_status & !(1<<2)) | ((coincidence as u8) << LCD_STATUS_COINCIDENCE);
		if coincidence && (*self.regs.lcd_status & (1<<LCD_STATUS_COINCIDENCE_INTERRUPT) != 0) {
			self.request_status_interrupt();
		}
	}

	//Getter and setter functions
	#[inline]
	pub fn get_obp0_palette(&self) -> u8 {
		*self.regs.obp0
	}

	#[inline]	
	pub fn get_obp1_palette(&self) -> u8 {
		*self.regs.obp1
	}
	
	#[inline]
	pub fn get_bg_palette(&self) -> u8 {
		*self.regs.bgp
	}
	
	#[inline]
	pub fn set_bg_palette(&mut self, data : u8) {
		*self.regs.bgp = data;
		VideoData::update_palette(&mut self.bg_palette, data);
	}
	
	#[inline]
	pub fn set_obp0_palette(&mut self, data : u8) {
		*self.regs.obp0 = data;
		VideoData::update_palette(&mut self.obp_palette[0], data);
	}

	#[inline]
	pub fn set_obp1_palette(&mut self, data : u8) {
		*self.regs.obp1 = data;
		VideoData::update_palette(&mut self.obp_palette[1], data);
	}
	
	fn update_palette(palette : &mut Palette, data : u8) {

		for c in 0..4 {
			palette[c] = match (data >> (2*c)) & 0b11 {
				1 => DMGColor::LightGray,
				2 => DMGColor::DarkGray,
				3 => DMGColor::Black,
				_ => DMGColor::White,
			};
		}
	}
	
	fn draw_line(&mut self) {
		let screen_y = *self.regs.ly as usize;
				
		let bb_row = &mut self.back_buffer[screen_y*160 ..(screen_y +1)*160]; 

		//draw background
		if self.lcd_ctrl.bg_enabled {
			let bgmap_index = self.lcd_ctrl.bg_tile_map_1_sel as usize;
			let bgmap = &self.vram0.tile_map[bgmap_index];
			let line = (*self.regs.ly).wrapping_add(*self.regs.scy);
			let (tile_y, tile_row) = ((line/8 )as usize, (line%8) as usize);
			let scx = *self.regs.scx;
			
			for screen_x in 0..160 {
				let x = (screen_x as u8).wrapping_add(scx);
				let (tile_x, tile_col) = ((x/8) as usize, (x%8) as usize);
				let ti = bgmap[(tile_y*32 + tile_x) as usize] as usize;
				let adj_ti = if self.lcd_ctrl.tile_data_1_sel { (256 + ((ti as i8) as i16)) as usize  } else { ti };
		
				let tile_data = self.vram0.tile_ram[adj_ti][tile_row as usize];
				let col_index = (((tile_data[1] >> (7-tile_col)) & 1) << 1) | ((tile_data[0] >> (7-tile_col)) & 1);

				bb_row[screen_x] = self.bg_palette[col_index as usize] as u8;
			}
		}
		
		//draw window
		if self.lcd_ctrl.window_enabled && (*self.regs.wy <= *self.regs.ly) {
			let wndmap_index = self.lcd_ctrl.window_tile_map_1_sel as usize;
			let wndmap = &self.vram0.tile_map[wndmap_index];
			
			let tile_y = ((*self.regs.ly - *self.regs.wy) / 8 )as usize;
			let tile_row = ((*self.regs.ly - *self.regs.wy) % 8 )as usize;
			
			let start_x = max(0,(*self.regs.wx as i16)-7) as u8;
			for screen_x in start_x..160 {
				let tile_x = ((screen_x.wrapping_sub((*self.regs.wx).wrapping_sub(7))) / 8 )as usize;
				let tile_col = ((screen_x.wrapping_sub(((*self.regs.wx).wrapping_sub(7)))) % 8 )as usize;
				
				let ti = wndmap[(tile_y*32 + tile_x) as usize] as usize;
				let adj_ti = if self.lcd_ctrl.tile_data_1_sel { (256 + ((ti as i8) as i16)) as usize  } else { ti };
		
				let tile_data = self.vram0.tile_ram[adj_ti][tile_row as usize];
				let col_index = (((tile_data[1] >> (7-tile_col)) & 1) << 1) | ((tile_data[0] >> (7-tile_col)) & 1);

				bb_row[screen_x as usize] = self.bg_palette[col_index as usize] as u8;
			} 
		}

		//draw sprites
		if self.lcd_ctrl.obj_enabled {
			
			let sprite_size = if self.lcd_ctrl.obj_size_8x16 { 16 } else { 8 };
			let ly = *self.regs.ly;
			//find sprites in line
			let mut line_sprites : Vec<&Sprite> = self.oam.sprite_ram.iter().filter(|&s| {
				ly.wrapping_sub(s.y) < sprite_size //NOTE: sprites are not affected by scy
			}).take(10).collect();
			line_sprites.sort_by(|a,b| { 
				if a.x == b.x { 
					Ordering::Equal
				} else {
					if a.x > b.x { Ordering::Less } else { Ordering::Greater } 
				}
			});
			//draw all sprites in line
			for s in &line_sprites {
				let mut tile_index = s.tile;
				if self.lcd_ctrl.obj_size_8x16 {
					tile_index &= 0xfe;
				}
				let mut sprite_row = ly.wrapping_sub(s.y);
				if sprite_row >= 8 {
					sprite_row -= 8;
					if !s.y_flip {
						tile_index = tile_index.wrapping_add(1)
					}
				} else if s.y_flip && self.lcd_ctrl.obj_size_8x16 {
					tile_index = tile_index.wrapping_add(1)
				}
				let adj_row = if s.y_flip { 7 - sprite_row } else { sprite_row } as usize;
				let tile_data = self.vram0.tile_ram[tile_index as usize][adj_row];
				
				for c in 0..8 {
					let adj_col = if s.x_flip { c } else { 7 - c};
					let col_index = (((tile_data[1] >> adj_col) & 1) << 1) | ((tile_data[0] >> adj_col) & 1);
					
					let screen_x = s.x.wrapping_add(c);
					if screen_x < 160 {
						//sprite col 0 and bg color 0 are transparent 
						if col_index != 0 && (!s.priority || (bb_row[screen_x as usize] == self.bg_palette[0] as u8)) { 
							bb_row[screen_x as usize] =  self.obp_palette[s.palette_1_sel as usize][col_index as usize] as u8;
						}
					} else {
						break
					}
				}
			}			
		}
		
	}
	
	#[inline(always)]
	fn request_status_interrupt(&mut self) {
		let mut iregs = self.interrupt_regs.borrow_mut();
		*iregs.iflags |= interrupt::INTERRUPT_LCD_STAT
	}
	
	#[inline(always)]
	fn set_mode(&mut self, mode : VideoMode) {
		self.mode = mode;
		*self.regs.lcd_status = (*self.regs.lcd_status & 0xfc) | mode as u8;
	}
}

impl VRAMBank {
	
	pub fn new() -> VRAMBank {
		VRAMBank {
			tile_ram : Box::new([[[0; 2]; 8]; 384]),
			tile_map : Box::new([[0; 1024]; 2]),
			accessible : true			
		}
	}
	
	#[inline]
	pub fn read(&mut self, addr: u16) -> u8 {
		if self.accessible {
			match addr>>8 {
				0x00 ... 0x17 => {
					let index = (addr >> 4) as usize;
					let tile_row = ((addr >> 1) & 0x7) as usize; 
					self.tile_ram[index][tile_row][(addr & 1) as usize]
				},
				0x18 ... 0x1f => {
					let map_index = ((addr >> 10) & 1) as usize;
					let tile_index = (addr & 0x3ff) as usize;  
					self.tile_map[map_index][tile_index]
				},
				_ => unreachable!()
			}
		} else {
			0xff
		}
	}
	
	#[inline]
	pub fn write(&mut self, addr: u16, data: u8) {
		if self.accessible {
			match addr>>8 {
				0x00 ... 0x17 => {
					let index = (addr >> 4) as usize;
					let tile_row = ((addr >> 1) & 0x7) as usize;
					self.tile_ram[index][tile_row][(addr & 1) as usize] = data;
				},
				0x18 ... 0x1f => {
					let map_index = ((addr >> 10) & 1) as usize;
					let tile_index = (addr & 0x3ff) as usize;  
					self.tile_map[map_index][tile_index] = data;
				},
				_ => unreachable!()
			}
		}
	}

}


impl OAM {
	
	pub fn new() -> OAM {
		OAM {
			sprite_ram : vec![Sprite { ..Default::default() }; OAM_NUM_SPRITES],
			accessible : true,
			dma_transfer: false,
			dma_addr : 0			
		}
	}
			
	#[inline(always)]
	pub fn read(&mut self, addr: u16) -> u8 {
		if self.accessible {
			let sprite : &Sprite = &self.sprite_ram[(addr >> 2) as usize];
			let index = addr & 0x3;			
			match index {
				0 => sprite.y.wrapping_add(16),
				1 => sprite.x.wrapping_add(8),
				2 => sprite.tile,
				_ => sprite.opt_data
			}					
		} else {
			0xff
		}
	}
	
	#[inline(always)]
	pub fn write(&mut self, addr: u16, data: u8) {
		if self.accessible {
			let sprite : &mut Sprite = &mut self.sprite_ram[(addr >> 2) as usize];
			let index = addr & 0x3;			
			match index {
				0 => sprite.y = data.wrapping_sub(16),
				1 => sprite.x = data.wrapping_sub(8),
				2 => sprite.tile = data,
				_ => {
					sprite.opt_data = data;
					sprite.priority = data & (1<<7) != 0;
					sprite.y_flip = data & (1<<6) != 0;
					sprite.x_flip = data & (1<<5) != 0;
					sprite.palette_1_sel = data & (1<<4) != 0;
				}
			}
		}
	}
	
	pub fn trigger_dma(&mut self, src: u8) {
		if src >= 0x80 && src <= 0xdf { //TODO is this correct?
			self.dma_addr = (src as u16) << 8;
			self.dma_transfer = true			
		}
	}
	
	pub fn dma_start_address(&self) -> u8 {
		(self.dma_addr >> 8) as u8
	}
}


impl LCDControlRegister {
	
	pub fn new() -> LCDControlRegister {
		let mut ctrl = LCDControlRegister { ..Default::default() };
		ctrl.write(0x83); //BG/OBJ ON, LCDC OPERATION
		ctrl
	}
	
	pub fn write(&mut self, data : u8) {
		self.data = data;
		self.enabled = data & (1<<7) != 0;
		self.window_tile_map_1_sel = data & (1<<6) != 0;
		self.window_enabled = data & (1<<5) != 0;
		self.tile_data_1_sel = data & (1<<4) == 0;
		self.bg_tile_map_1_sel = data & (1<<3) != 0;
		self.obj_size_8x16 = data & (1<<2) != 0;
		self.obj_enabled = data & (1<<1) != 0;
		self.bg_enabled = data & (1<<0) != 0;
	}
	
	pub fn read(&self) -> u8 {
		self.data
	}
}

