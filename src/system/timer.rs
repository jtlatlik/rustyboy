use std::rc::Rc;
use std::cell::RefCell;

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
const MUX_LUT : [u16; 4] = [9, 3, 5 ,7]; 

pub struct DividerRegister(pub u16);

pub struct TimerRegisters {
	divider : u16,
	counter : u16,
	pub modulo : IORegister,
	pub control : IORegister,
	counter_cycles : u32,
	interrupt_regs : Rc<RefCell<InterruptRegisters>>
}

impl TimerRegisters {
	pub fn update(&mut self, delta : u32) {
		
		self.update_cycle_by_cycle(delta)
		/*		
		self.divider.0 = self.divider.0.wrapping_add(delta as u16);
		self.counter_cycles += delta;
		
		let timer_enabled = (*self.control & TIMER_ENABLE_MASK) != 0;
		let max_cycles = TIMER_LUT[(*self.control & CLK_INPUT_MASK) as usize];
		if timer_enabled && (self.counter_cycles >= max_cycles) {
			self.counter_cycles %= max_cycles;
			if *self.counter == 255 {
				//generate timer overflow interrupt request
				let mut iregs = self.interrupt_regs.borrow_mut();
				*iregs.iflags = *iregs.iflags | interrupt::INTERRUPT_TIMER; 
				
				*self.counter = *self.modulo;
			} else {
				*self.counter += 1;
			}
		}*/

	}
	
	fn update_cycle_by_cycle(&mut self, delta : u32) {
		
		let timer_enabled = (*self.control & TIMER_ENABLE_MASK) != 0;

		for _ in 0..delta {
			let next_divider = self.divider.wrapping_add(1);
			
			if timer_enabled && self.falling_edge(next_divider) {
				
				self.increase_counter();
			};
			self.divider = next_divider;
		}

	}
	
	
	fn falling_edge(&self, new : u16) -> bool {
		
		let mux_a = (*self.control & CLK_INPUT_MASK) as usize;
		let h2l_bits = self.divider & !new;
		
		h2l_bits & (1<<MUX_LUT[mux_a]) != 0
	}
	
	fn increase_counter(&mut self) {

		self.counter += 1;
		if self.counter == 256 {
			//generate timer overflow interrupt request
			let mut iregs = self.interrupt_regs.borrow_mut();
			*iregs.iflags = *iregs.iflags | interrupt::INTERRUPT_TIMER;
			
			self.counter = *self.modulo as u16;
		}
	}
	
	
	
	pub fn new(iregs : Rc<RefCell<InterruptRegisters>>) -> TimerRegisters {
		TimerRegisters {
			divider : 0,
			counter : 0,
			modulo : IORegister::new(),
			control : IORegister::new().write_mask(bits!(2 1 0)),
			counter_cycles : 0,
			interrupt_regs : iregs,
		}
	}
	
	pub fn write_counter(&mut self, value : u8) {
		self.counter = value as u16;
	}
	
	pub fn read_counter(&self) -> u8 {
		self.counter as u8
	}
	
	pub fn clear_divider(&mut self) {
		//When writing to DIV, if the current output is '1' and timer is enabled, as the new value after reseting DIV will be '0', 
		//the falling edge detector will detect a falling edge and TIMA will increase.
		let timer_enabled = (*self.control & TIMER_ENABLE_MASK) != 0;
		let next_divider = 0 as u16;
		if timer_enabled && self.falling_edge(0) {
			self.increase_counter();
		}
		self.divider = next_divider;
	}
	
	pub fn read_divider(&self) -> u8 {
		(self.divider >> 8) as u8
	}
}