use std::io::Write;

use std::rc::Rc;
use std::cell::RefCell;

use std::sync::mpsc::{self, Receiver, Sender};
use core::memory::Memory;
use rom::Rom;

use super::ioregister::IORegister;
use super::mbc::MBC;
use super::video::{VideoData, VRAMBank, OAM};
use super::sound::SoundData;
use super::timer::TimerRegisters;
use super::interrupt::InterruptRegisters;
use super::serial::SerialRegisters;
use super::wram::*;
use super::joypad::Joypad;


macro_rules! bits {
	( $($bit:expr)* ) => ( 0x00 $( | (1<<$bit) )* )
}

pub trait MemoryAccess {
	fn write(&mut self, addr: u16, data: u8);
	fn read(&mut self, addr: u16) -> u8;
}

struct IODummy;

impl MemoryAccess for IODummy {
	fn write(&mut self, _: u16, _: u8) {  }
	fn read(&mut self, _: u16) -> u8 { 0xff }
}
/*
impl<T: MemoryAccess> MemoryAccess for Arc<RwLock<T>> {

	fn write(&mut self, addr: u16, data: u8) {
		//acquire write lock and write data
		let lock =RwLock::write(&self).unwrap();
		lock.write(addr, data);
	}
	fn read(&mut self, addr: u16) -> u8 {
		//acquire read lock and read data
		let lock =RwLock::read(&self).unwrap();
		lock.read(addr)
	}
	
}*/

pub struct GBSystem {

	pub mbc	: MBC,
	pub video : VideoData,
	pub sound : SoundData,
	pub interrupt_regs : Rc<RefCell<InterruptRegisters>>,
	pub joypad : Joypad,
		
	wram0 : WRAMBank,
	wram1 : WRAMBank,
	timer_regs : TimerRegisters,
	serial_regs : SerialRegisters,
	
	zero_page : ZeroPageRAM,
	dummy : IODummy,
}

impl GBSystem {
		
	pub fn new(rom : Rom) -> GBSystem {
		
		//generate shared iregs instance first
		let iregs = Rc::new(RefCell::new(InterruptRegisters{ ..Default::default() }));
		
		GBSystem {
			mbc : MBC::new(rom),
			wram0 : WRAMBank(Box::new([0; WRAM_BANK_SIZE])),
			wram1 : WRAMBank(Box::new([0; WRAM_BANK_SIZE])),
			video : VideoData::new(iregs.clone()),
			interrupt_regs : iregs.clone(),
			timer_regs : TimerRegisters::new(iregs.clone()),
			sound : SoundData{ ..Default::default() },
			serial_regs : SerialRegisters{ ..Default::default() },
			zero_page : ZeroPageRAM(Box::new([0; 128])),
			joypad: Joypad::new(iregs.clone()),
			dummy: IODummy
		}
	}
	
	pub fn update(&mut self, delta: u32) {
		
		self.video.update(delta);
		self.timer_regs.update(delta);
		
		for _ in 0..(delta/4) {
			if self.video.oam.dma_transfer {
				let addr = self.video.oam.dma_addr;
				let data = self.read8(addr);
				let index = addr & 0xff;
				self.video.oam.write(index, data);
				if index + 1 == 0xa0 {
					self.video.oam.dma_transfer = false;
				}
				self.video.oam.dma_addr = addr+1;
			}
		}
	}


    pub fn read8(&mut self, addr: u16) -> u8 {
    	let addr_l = addr as u8;
    	let addr_h = (addr >> 8) as u8;
		match addr_h {
			0x00 ... 0x7f => self.mbc.read(addr),
			0x80 ... 0x9f => self.video.vram0.read(addr - 0x8000),
			0xa0 ... 0xbf => self.mbc.read(addr), 
			0xc0 ... 0xcf => self.wram0.read(addr - 0xc000),
			0xd0 ... 0xdf => self.wram1.read(addr - 0xd000),
			0xe0 ... 0xef => self.wram0.read(addr - 0xe000),
			0xf0 ... 0xfd => self.wram1.read(addr - 0xf000),
			0xfe => match addr_l {
				0x00 ... 0x9f => self.video.oam.read(addr - 0xfe00),
				0xa0 ... 0xff => self.dummy.read(addr),
				_ => unreachable!()
			},
			0xff => match addr_l {
				0x00 => self.joypad.get_register(), 					//JOYPAD
				0x01 => self.serial_regs.data.read(addr),		//SB
				0x02 => self.serial_regs.control.read(addr),		//SC
				0x04 => self.timer_regs.read_divider(),
				0x05 => self.timer_regs.read_counter(),
				0x06 => self.timer_regs.modulo.read(addr),
				0x07 => self.timer_regs.control.read(addr),
				
	    		0x0f => self.interrupt_regs.borrow_mut().iflags.read(addr),
				0x10 => self.sound.regs.ch1_sweep.read(addr),
				0x11 => self.sound.regs.ch1_length_duty.read(addr),
				0x12 => self.sound.regs.ch1_vol_env.read(addr),
				0x13 => self.sound.regs.ch1_freq_low.read(addr),
				0x14 => self.sound.regs.ch1_freq_high.read(addr),
				
				0x16 => self.sound.regs.ch2_length_duty.read(addr),
				0x17 => self.sound.regs.ch2_vol_env.read(addr),
				0x18 => self.sound.regs.ch2_freq_low.read(addr),
				0x19 => self.sound.regs.ch2_freq_high.read(addr),
				0x1a => self.sound.regs.ch3_snd_on_off.read(addr),
				0x1b => self.sound.regs.ch3_snd_length.read(addr),
				0x1c => self.sound.regs.ch3_out_level.read(addr),
				0x1d => self.sound.regs.ch3_freq_low.read(addr),
				0x1e => self.sound.regs.ch3_freq_high.read(addr),
				
				0x20 => self.sound.regs.ch4_snd_length.read(addr),
				0x21 => self.sound.regs.ch4_vol_env.read(addr),
				0x22 => self.sound.regs.ch4_poly_cnt.read(addr),
				0x23 => self.sound.regs.ch4_cnt_init.read(addr),
				0x24 => self.sound.regs.ctrl_vol.read(addr),
				0x25 => self.sound.regs.ctrl_ch_mux.read(addr),
				0x26 => self.sound.regs.ctrl_on_off.read(addr),
				
				0x30 ... 0x3f => self.sound.wave_ram.read(addr - 0xff30),
				0x40 => self.video.lcd_ctrl.read(),									// LCDC
				0x41 => self.video.regs.lcd_status.read(addr),						// STAT
				0x42 => self.video.regs.scy.read(addr),								// SCY
				0x43 => self.video.regs.scx.read(addr),								// SCX
				0x44 => self.video.regs.ly.read(addr),								// LY
				0x45 => self.video.regs.lyc.read(addr),								// LYC
				0x46 => self.video.oam.dma_start_address(),							// OAM DMA
				0x47 => self.video.get_bg_palette(),								// BGP
				0x48 => self.video.get_obp0_palette(),								// OBP0
				0x49 => self.video.get_obp1_palette(),								// OBP1
				0x4a => self.video.regs.wy.read(addr),								// WY
				0x4b => self.video.regs.wx.read(addr),								// Wx
				
				0x80 ... 0xfe => self.zero_page.read(addr - 0xff80),			// Zero Page RAM
				0xff => self.interrupt_regs.borrow_mut().ienable.read(addr),			// IE
				_ => self.dummy.read(addr)
			},
			_ => unreachable!()
		}
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
    	let addr_l = addr as u8;
    	let addr_h = (addr >> 8) as u8;
		match addr_h {
			0x00 ... 0x7f => self.mbc.write(addr, data),
			0x80 ... 0x9f => self.video.vram0.write(addr - 0x8000, data),
			0xa0 ... 0xbf => self.mbc.write(addr, data), 
			0xc0 ... 0xcf => self.wram0.write(addr - 0xc000, data),
			0xd0 ... 0xdf => self.wram1.write(addr - 0xd000, data),
			0xe0 ... 0xef => self.wram0.write(addr - 0xe000, data),
			0xf0 ... 0xfd => self.wram1.write(addr - 0xf000, data),
			0xfe => match addr_l {
				0x00 ... 0x9f => self.video.oam.write(addr - 0xfe00,data),
				0xa0 ... 0xff => self.dummy.write(addr,data),
				_ => unreachable!()
			},
			0xff => match addr_l {
				0x00 => self.joypad.set_register(data),					//JOYPAD
				0x01 => self.serial_regs.data.write(addr, data),		//SB
				0x02 => self.serial_regs.control.write(addr, data),		//SC
				0x04 => self.timer_regs.clear_divider(),
				0x05 => self.timer_regs.write_counter(data),
				0x06 => self.timer_regs.modulo.write(addr, data),
				0x07 => self.timer_regs.control.write(addr, data),
				
	    		0x0f => self.interrupt_regs.borrow_mut().iflags.write(addr, data),
				0x10 => self.sound.regs.ch1_sweep.write(addr, data),
				0x11 => self.sound.regs.ch1_length_duty.write(addr, data),
				0x12 => self.sound.regs.ch1_vol_env.write(addr, data),
				0x13 => self.sound.regs.ch1_freq_low.write(addr, data),
				0x14 => self.sound.regs.ch1_freq_high.write(addr, data),
				
				0x16 => self.sound.regs.ch2_length_duty.write(addr, data),
				0x17 => self.sound.regs.ch2_vol_env.write(addr, data),
				0x18 => self.sound.regs.ch2_freq_low.write(addr, data),
				0x19 => self.sound.regs.ch2_freq_high.write(addr, data),
				0x1a => self.sound.regs.ch3_snd_on_off.write(addr, data),
				0x1b => self.sound.regs.ch3_snd_length.write(addr, data),
				0x1c => self.sound.regs.ch3_out_level.write(addr, data),
				0x1d => self.sound.regs.ch3_freq_low.write(addr, data),
				0x1e => self.sound.regs.ch3_freq_high.write(addr, data),
				
				0x20 => self.sound.regs.ch4_snd_length.write(addr, data),
				0x21 => self.sound.regs.ch4_vol_env.write(addr, data),
				0x22 => self.sound.regs.ch4_poly_cnt.write(addr, data),
				0x23 => self.sound.regs.ch4_cnt_init.write(addr, data),
				0x24 => self.sound.regs.ctrl_vol.write(addr, data),
				0x25 => self.sound.regs.ctrl_ch_mux.write(addr, data),
				0x26 => self.sound.regs.ctrl_on_off.write(addr, data),
				
				0x30 ... 0x3f => self.sound.wave_ram.write(addr - 0xff30, data),
				0x40 => self.video.lcd_ctrl.write(data),									// LCDC
				0x41 => self.video.regs.lcd_status.write(addr, data),						// STAT
				0x42 => self.video.regs.scy.write(addr, data),								// SCY
				0x43 => self.video.regs.scx.write(addr, data),								// SCX
				0x44 => self.video.regs.ly.write(addr, data),								// LY
				0x45 => self.video.regs.lyc.write(addr, data),								// LYC
				0x46 => self.video.oam.trigger_dma(data),									// OAM DMA transfer here
				0x47 => self.video.set_bg_palette(data),									// BGP
				0x48 => self.video.set_obp0_palette(data),									// OBP0
				0x49 => self.video.set_obp1_palette(data),									// OBP1
				0x4a => self.video.regs.wy.write(addr, data),								// WY
				0x4b => self.video.regs.wx.write(addr, data),								// Wx
				
				0x80 ... 0xfe => self.zero_page.write(addr - 0xff80, data),			// Zero Page RAM
				0xff => self.interrupt_regs.borrow_mut().ienable.write(addr, data),			// IE
				_ => self.dummy.write(addr,data)
			},
			_ => unreachable!()
		}
    }
       
    pub fn read16(&mut self, addr: u16) -> u16 {
    	((self.read8(addr.wrapping_add(1)) as u16) << 8) | (self.read8(addr) as u16)
    }
    
    pub fn write16(&mut self, addr: u16, data: u16) {
    	self.write8(addr, data as u8);
    	self.write8(addr.wrapping_add(1), (data >> 8) as u8)
    }
}
