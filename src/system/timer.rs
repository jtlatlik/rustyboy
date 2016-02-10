use std::sync::{Arc, RwLock};

use super::system::MemoryAccess;
use super::ioregister::IORegister;
use super::interrupt::{self, InterruptRegisters};

macro_rules! bits {
	( $($bit:expr)* ) => ( 0x00 $( | (1<<$bit) )* )
}

const CLK_DIVIDER :u32 = 256;
const TIMER_ENABLE_MASK : u8 = 0x04;
const CLK_INPUT_MASK : u8 = 0x3;

const TIMER_LUT : [u32;4] = [1024, 16, 64, 256];

pub struct DividerRegister(pub u8);

pub struct TimerRegisters {
	pub divider : DividerRegister,
	pub counter : IORegister,
	pub modulo : IORegister,
	pub control : IORegister,
	divider_cycles : u32,
	counter_cycles : u32,
	interrupt_regs : Arc<RwLock<InterruptRegisters>>
}

impl TimerRegisters {
	pub fn update(&mut self, delta : u32) {
		
		self.divider_cycles += delta;
		self.counter_cycles += delta;
		
		if self.divider_cycles >= CLK_DIVIDER {
			self.divider_cycles -= CLK_DIVIDER;
			self.divider.0 = self.divider.0.wrapping_add(1)
		}
		
		let timer_enabled = (*self.control & TIMER_ENABLE_MASK) != 0;
		let max_cycles = TIMER_LUT[(*self.control & CLK_INPUT_MASK) as usize];
		if timer_enabled && (self.counter_cycles >= max_cycles) {
			self.counter_cycles %= max_cycles;
			if *self.counter == 255 {
				//generate timer overflow interrupt request
				let mut iregs = self.interrupt_regs.write().unwrap();
				*iregs.iflags = *iregs.iflags | interrupt::INTERRUPT_TIMER; 
				
				*self.counter = *self.modulo;
			} else {
				*self.counter += 1;
			}
		}

	}
	
	pub fn new(iregs : Arc<RwLock<InterruptRegisters>>) -> TimerRegisters {
		TimerRegisters {
			divider : DividerRegister(0),
			counter : IORegister::new(),
			modulo : IORegister::new(),
			control : IORegister::new().write_mask(bits!(2 1 0)),
			divider_cycles : 0,
			counter_cycles : 0,
			interrupt_regs : iregs
		}
	}
}

impl MemoryAccess for DividerRegister {
	
	//any write to divider register resets it.
	fn write(&mut self, _: u16, _: u8) {
		self.0 = 0
	}
	
	fn read(&mut self, _: u16) -> u8 {
		self.0
	}
}