use std::sync::Arc;
use std::cell::RefCell;

use super::gb::GBRegisters;
use core::operands::Reg16Operand;
use super::memory::*;
use system::system::GBSystem;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use super::register::Contents;

use time;

pub struct CPU {

    pub regs : GBRegisters,
    pub sys : Arc<RefCell<GBSystem>>,
    pub halt_mode : bool,
    pub stop_mode : bool,
    
    pub clk_period_ns : f64,
    
    pub trace_file : Option<File>
}

impl CPU {
	
	pub fn new(sys : Arc<RefCell<GBSystem>>) -> CPU {
	    let mut cpu = CPU {
	        regs : GBRegisters::new(),
	        sys : sys,
	        halt_mode : false,
	        stop_mode : false,
	        clk_period_ns : 238.418579,
	        trace_file : None
	    };
	    cpu.reset();
	    cpu
	}
	
	pub fn set_trace_file(&mut self, trace_file: File) {
		self.trace_file = Some(trace_file);
	}
	
	pub fn fetch(&mut self, addr: u16) -> [u8; 3] {
		let mut mem = self.sys.borrow_mut();
		[mem.read8(addr), mem.read8(addr.wrapping_add(1)), mem.read8(addr.wrapping_add(2))]
	} 
	
	//run the next instruction
	pub fn run_instruction(&mut self) -> f64 {
		
		let mut delta_cycles = 4;
		if !(self.halt_mode || self.stop_mode) {
			//fetch instruction
			let pc = self.regs.pc;
			let insn_bytes = self.fetch(pc);

			//decode insn
			let insn = self.decode(insn_bytes);
			
			//print raw and decoded instruction
			if let Some(ref mut tracefile) = self.trace_file {
				let mut trace_line = String::with_capacity(128);
				//print register contents and pc
				trace_line.push_str(&format!("{} | {:04x}:", self.regs, pc));
				for i in 0..3 {
					if i < insn.length {
						trace_line.push_str(&format!(" {:>02x}", insn_bytes[i as usize]));
					} else {
						trace_line.push_str("   ");
					}
				}
				trace_line.push_str(&format!(" {}\n", insn));	
				tracefile.write_all(trace_line.as_bytes()).unwrap();
			}
			//execute insn
			delta_cycles = match self.execute(insn) {
				Ok(c) => c,
				Err(e) => {
					panic!("\n Fatal error during execution: {:?}", e);
				}
			}
		}
		//update periphery
		self.sys.borrow_mut().update(delta_cycles);
		
		//handle interrupts
		let interrupt_cycles = match self.handle_interrupts() {
			Some(c) => {
				//update again if necessary
				self.sys.borrow_mut().update(c);
				c
			},
			None => 0
		};
		//return simulation time
		self.clk_period_ns * ((delta_cycles + interrupt_cycles) as f64)
	}
	
	pub fn reset(&mut self) {
		self.regs.pc = 0x100;
	    self.regs.set16(Reg16Operand::af, 0x01b0);
	    self.regs.bc = 0x0013;
	    self.regs.de = 0x00d8;
	    self.regs.hl = 0x014d;
	    self.regs.sp = 0xfffe;
	    self.regs.ime = false;
	}
}
