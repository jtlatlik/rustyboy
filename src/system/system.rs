use std::process;
use std::fs::*;
use std::io::Write;

use std::rc::Rc;
use std::rc::Weak;

use std::option::Option;

use core::cpu::CPU;
use core::memory::Memory;

use rom::*;

use super::video::*;
use super::sound::*;
use super::timer::*;

const WRITE_ALL: u8 = 0xff;
const WRITE_NONE: u8 = 0xff;

macro_rules! writable {
	( $($bit:expr)+ ) => { ( 0x00 $( | (1 << $bit) )+ ) } 
}

const WRAM_BANK_SIZE :usize = 4*1024; //4K per WRAM bank

type WRAMBank = Box<[u8; WRAM_BANK_SIZE]>;

#[derive(Default)]
pub struct InterruptRegisters {
	pub iflags : u8,
	pub iswitch : u8
}

pub struct GBSystem {

	pub rom	: Rom,
	pub video : VideoData,
	
	wram_bank0 : WRAMBank,
	wram_bankn : WRAMBank,
	
	pub interrupt_regs : InterruptRegisters,
	
	timer_regs : TimerRegisters,
	zero_page : [u8; 128],
	serial_data : u8,
	
	wave_ram : [u8; 16],
	sound_regs : SoundRegisters,
	dummy : u8
	
}

impl GBSystem {
		
	pub fn new(rom : Rom) -> GBSystem {
		GBSystem {
			rom : rom,
			wram_bank0 : Box::new([0; WRAM_BANK_SIZE]),
			wram_bankn : Box::new([0; WRAM_BANK_SIZE]),
			video : VideoData{ ..Default::default() },
			interrupt_regs : InterruptRegisters{ ..Default::default() },
			timer_regs : TimerRegisters{ ..Default::default() },
			sound_regs : SoundRegisters{ ..Default::default() },
			zero_page : [0; 128],
			serial_data : 0,
			wave_ram : [0; 16],
			dummy: 0xff
		}
	} 
	
	pub fn update(&mut self, delta: u32) {
		self.video.update(delta)
	}
	
	fn decode_addr(&mut self, addr: u16) -> (&mut u8, u8) {
		let off14 = (addr & 0x3fff) as usize;
		let off13 = (addr & 0x1fff) as usize;
		let off12 = (addr & 0x0fff) as usize;
		let off8 = (addr & 0x00ff) as usize;
		let off7 = (addr & 0x007f) as usize;
		let off4 = (addr & 0x000f) as usize;

		match addr {
			0x0000 ... 0x00ff => (&mut self.rom.banks[0][off14], WRITE_NONE),//ROM bank 0 or startup code
			0x0100 ... 0x3fff => (&mut self.rom.banks[0][off14], WRITE_NONE),
			0x4000 ... 0x7fff => (&mut self.rom.banks[1][off14], WRITE_NONE),
			0x8000 ... 0x9fff => (&mut self.video.vram0[off13], WRITE_ALL),
			0xa000 ... 0xbfff => unimplemented!(),							// EXT RAM
			0xc000 ... 0xcfff => (&mut self.wram_bank0[off12], WRITE_ALL),	// Working RAM Bank0
			0xd000 ... 0xdfff => (&mut self.wram_bankn[off12], WRITE_ALL),	// Working RAM Bank1-N
			0xe000 ... 0xefff => (&mut self.wram_bank0[off12], WRITE_ALL),	// Working RAM Bank0 shadow
			0xf000 ... 0xfdff => (&mut self.wram_bankn[off12], WRITE_ALL),	// Working RAM Bank1-N shadow
			0xfe00 ... 0xfe9f => (&mut self.video.oam[off8], WRITE_ALL),   	// OAM
			0xfea0 ... 0xfeff => (&mut self.dummy, WRITE_NONE),				// Not usable
			
			0xff01 => (&mut self.serial_data, WRITE_ALL),
			0xff02 => (&mut self.dummy, WRITE_NONE),
			/*
			0xff04 => (&mut self.timer_regs.divider, WRITE_ALL),			// DIV
			0xff05 => (&mut self.timer_regs.counter, WRITE_ALL),			// TIMA
			0xff06 => (&mut self.timer_regs.modulo, WRITE_ALL),				// TMA
			0xff07 => (&mut self.timer_regs.control, writable!(2 1 0)),		// TAC
			*/
			0xff0f => (&mut self.interrupt_regs.iflags, WRITE_ALL),			// IFLAGS [RW] Interrupt Flags
			
			0xff10 => (&mut self.sound_regs.ch1_sweep, WRITE_ALL),
			0xff11 => (&mut self.sound_regs.ch1_length_duty, WRITE_ALL),
			0xff12 => (&mut self.sound_regs.ch1_vol_env, WRITE_ALL),
			0xff13 => (&mut self.sound_regs.ch1_freq_low, WRITE_ALL),
			0xff14 => (&mut self.sound_regs.ch1_freq_high, WRITE_ALL),
			
			0xff16 => (&mut self.sound_regs.ch2_length_duty, WRITE_ALL),
			0xff17 => (&mut self.sound_regs.ch2_vol_env, WRITE_ALL),
			0xff18 => (&mut self.sound_regs.ch2_freq_low, WRITE_ALL),
			0xff19 => (&mut self.sound_regs.ch2_freq_high, WRITE_ALL),
			
			0xff1a => (&mut self.sound_regs.ch3_snd_on_off, WRITE_ALL),
			0xff1b => (&mut self.sound_regs.ch3_snd_length, WRITE_ALL),
			0xff1c => (&mut self.sound_regs.ch3_out_level, WRITE_ALL),
			0xff1d => (&mut self.sound_regs.ch3_freq_low, WRITE_ALL),
			0xff1e => (&mut self.sound_regs.ch3_freq_high, WRITE_ALL),
			
			0xff20 => (&mut self.sound_regs.ch4_snd_length, WRITE_ALL),
			0xff21 => (&mut self.sound_regs.ch4_vol_env, WRITE_ALL),
			0xff22 => (&mut self.sound_regs.ch4_poly_cnt, WRITE_ALL),
			0xff23 => (&mut self.sound_regs.ch4_cnt_init, WRITE_ALL),
			
			0xff24 => (&mut self.sound_regs.ctrl_vol, WRITE_ALL),
			0xff25 => (&mut self.sound_regs.ctrl_ch_mux, WRITE_ALL),
			0xff26 => (&mut self.sound_regs.ctrl_on_off, WRITE_ALL),

			0xff30 ... 0xff3f => (&mut self.wave_ram[off4], WRITE_ALL),
			
			0xff40 => (&mut self.video.regs.lcd_ctrl, WRITE_ALL),			// LCDC
			0xff41 => (&mut self.video.regs.lcd_status, writable!(6 5 4 3)),// STAT
			0xff42 => (&mut self.video.regs.scy, WRITE_ALL),				// SCY
			0xff43 => (&mut self.video.regs.scx, WRITE_ALL),				// SCX
			0xff44 => (&mut self.video.regs.ly, WRITE_NONE),				// LY
			0xff45 => (&mut self.video.regs.lyc, WRITE_ALL),				// LYC
			
			0xff47 => (&mut self.video.regs.bgp, WRITE_ALL),				// BGP
			0xff48 => (&mut self.video.regs.obp0, WRITE_ALL),				// OBP0
			0xff49 => (&mut self.video.regs.obp1, WRITE_ALL),				// OBP1
			0xff4a => (&mut self.video.regs.wy, WRITE_ALL),					// WY
			0xff4b => (&mut self.video.regs.wx, WRITE_ALL),					// Wx
			
			0xff80 ... 0xfffe => (&mut self.zero_page[off7], WRITE_ALL),	// Zero Page RAM
			0xffff => (&mut self.interrupt_regs.iswitch, WRITE_ALL),		// ISWITCH [RW] Interrupt Enable/Disable
			_ => (&mut self.dummy, WRITE_NONE)
		}
	}

    pub fn read8(&mut self, addr: u16) -> u8 {
		let (dest, _) = self.decode_addr(addr);
		*dest
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
    	print!("(0x{:04x}) = 0x{:>02x}", addr, data);
    	{
			let (dest, mask) = self.decode_addr(addr);
			*dest = *dest & !mask | data & mask;
    	}
    	
		if addr == 0xff02 {
			let mut file = match OpenOptions::new().write(true).append(true).open("serial.txt") {
				Ok(f) => f,
				Err(e) => panic!("error creating file!")
			};
			
			file.write_all(&[self.serial_data]);
		}
    }
    
    pub fn read16(&mut self, addr: u16) -> u16 {
    	((self.read8(addr+1) as u16) << 8) | (self.read8(addr) as u16)
    }
    
    pub fn write16(&mut self, addr: u16, data: u16) {
    	self.write8(addr, data as u8);
    	self.write8(addr+1, (data >> 8) as u8)
    }
}