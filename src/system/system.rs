use std::io::Write;

use core::memory::Memory;
use rom::Rom;

use super::ioregister::IORegister;
use super::mbc::MBC;
use super::video::VideoData;
use super::sound::SoundData;
use super::timer::TimerRegisters;
use super::interrupt::InterruptRegisters;
use super::serial::SerialRegisters;
use super::wram::*;

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

pub struct GBSystem {

	pub mbc	: MBC,
	pub video : VideoData,
	pub sound : SoundData,
	
	wram0 : WRAMBank,
	wram1 : WRAMBank,
	
	pub interrupt_regs : InterruptRegisters,
	timer_regs : TimerRegisters,
	serial_regs : SerialRegisters,
	joypad : IORegister,
	zero_page : ZeroPageRAM,
	
	dummy : IODummy,
}

impl GBSystem {
		
	pub fn new(rom : Rom) -> GBSystem {

		GBSystem {
			mbc : MBC::new(rom),
			wram0 : WRAMBank(Box::new([0; WRAM_BANK_SIZE])),
			wram1 : WRAMBank(Box::new([0; WRAM_BANK_SIZE])),
			video : VideoData{ ..Default::default() },
			interrupt_regs : InterruptRegisters{ ..Default::default() },
			timer_regs : TimerRegisters{ ..Default::default() },
			sound : SoundData{ ..Default::default() },
			serial_regs : SerialRegisters{ ..Default::default() },
			zero_page : ZeroPageRAM(Box::new([0; 128])),
			joypad: IORegister::new().write_mask(bits!(5 4)),
			dummy: IODummy,
			
		}
	} 
	
	pub fn update(&mut self, delta: u32) {
		self.video.update(delta)
	}


    pub fn read8(&mut self, addr: u16) -> u8 {
    	let(target,offset) = self.decode(addr); 
		target.read(offset)
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
		print!("(0x{:04x}) = 0x{:>02x}", addr, data);
		let(target,offset) = self.decode(addr); 
		target.write(offset, data)
    }
    
	fn decode(&mut self, addr: u16) -> (&mut MemoryAccess, u16) {

		match addr {
    		0x0000 ... 0x7fff => (&mut self.mbc, addr),				// ROM
    		0x8000 ... 0x9fff => (&mut self.video, addr),			// VRAM
    		0xa000 ... 0xbfff => (&mut self.mbc, addr),				// EXT RAM
    		0xc000 ... 0xcfff => (&mut self.wram0, addr&0xfff),		// WRAM bank0
    		0xd000 ... 0xdfff => (&mut self.wram1, addr&0xfff),		// WRAM bank1
    		0xe000 ... 0xefff => (&mut self.wram0, addr&0xfff),		// WRAM bank0 shadow
    		0xf000 ... 0xfdff => (&mut self.wram1, addr&0xfff),		// WRAM bank1 shadow
    		0xfe00 ... 0xfe9f => (&mut self.video, addr),			// OAM
    		0xfea0 ... 0xfeff => (&mut self.dummy, 0), 				// Not usable
    		
    		0xff00 => (&mut self.joypad, 0),
			0xff01 => (&mut self.serial_regs.data, 0),				// SB
			0xff02 => (&mut self.serial_regs.control, 0),			// SC
    		
			0xff04 => (&mut self.timer_regs.divider, 0),			// DIV
			0xff05 => (&mut self.timer_regs.counter, 0),			// TIMA
			0xff06 => (&mut self.timer_regs.modulo, 0),				// TMA
			0xff07 => (&mut self.timer_regs.control, 0),			// TAC
    		
    		0xff0f => (&mut self.interrupt_regs.iflags, 0),			// IF
			0xff10 => (&mut self.sound.regs.ch1_sweep, 0),
			0xff11 => (&mut self.sound.regs.ch1_length_duty, 0),
			0xff12 => (&mut self.sound.regs.ch1_vol_env, 0),
			0xff13 => (&mut self.sound.regs.ch1_freq_low, 0),
			0xff14 => (&mut self.sound.regs.ch1_freq_high, 0),
			
			0xff16 => (&mut self.sound.regs.ch2_length_duty, 0),
			0xff17 => (&mut self.sound.regs.ch2_vol_env, 0),
			0xff18 => (&mut self.sound.regs.ch2_freq_low, 0),
			0xff19 => (&mut self.sound.regs.ch2_freq_high, 0),
			0xff1a => (&mut self.sound.regs.ch3_snd_on_off, 0),
			0xff1b => (&mut self.sound.regs.ch3_snd_length, 0),
			0xff1c => (&mut self.sound.regs.ch3_out_level, 0),
			0xff1d => (&mut self.sound.regs.ch3_freq_low, 0),
			0xff1e => (&mut self.sound.regs.ch3_freq_high, 0),
			
			0xff20 => (&mut self.sound.regs.ch4_snd_length, 0),
			0xff21 => (&mut self.sound.regs.ch4_vol_env, 0),
			0xff22 => (&mut self.sound.regs.ch4_poly_cnt, 0),
			0xff23 => (&mut self.sound.regs.ch4_cnt_init, 0),
			0xff24 => (&mut self.sound.regs.ctrl_vol, 0),
			0xff25 => (&mut self.sound.regs.ctrl_ch_mux, 0),
			0xff26 => (&mut self.sound.regs.ctrl_on_off, 0),
			
			0xff30 ... 0xff3f => (&mut self.sound.wave_ram, addr & 0xf),
			0xff40 => (&mut self.video.regs.lcd_ctrl, 0),			// LCDC
			0xff41 => (&mut self.video.regs.lcd_status, 0),			// STAT
			0xff42 => (&mut self.video.regs.scy, 0),				// SCY
			0xff43 => (&mut self.video.regs.scx, 0),				// SCX
			0xff44 => (&mut self.video.regs.ly, 0),					// LY
			0xff45 => (&mut self.video.regs.lyc, 0),				// LYC
			
			0xff47 => (&mut self.video.regs.bgp, 0),				// BGP
			0xff48 => (&mut self.video.regs.obp0, 0),				// OBP0
			0xff49 => (&mut self.video.regs.obp1, 0),				// OBP1
			0xff4a => (&mut self.video.regs.wy, 0),					// WY
			0xff4b => (&mut self.video.regs.wx, 0),					// Wx
			
    		0xff80 ... 0xfffe => (&mut self.zero_page, addr&0x7f),	// Zero Page RAM
    		0xffff => (&mut self.interrupt_regs.ienable, 0),		// IE
    		_ => (&mut self.dummy, 0)
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