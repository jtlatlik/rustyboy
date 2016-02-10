extern crate readline;

use std::sync::{Arc, RwLock};
use std::cell::RefCell;
use std::io::{self, Write};
use std::ffi::{CStr, CString};

use core::cpu::CPU;
use core::operands::{Reg8Operand, Reg16Operand};
use system::system::GBSystem;
use rom::*;

macro_rules! extract_arg {
	($tok:ident, $i:expr, $name:expr) => {
		if ($tok.len() >= $i + 1) && ($tok[$i].len() > 0) {
			$tok[$i]
		} else { 
			println!(concat!("missing argument: ", $name));
			continue
		}
	}
}

macro_rules! extract_opt_arg {
	($tok:ident, $i:expr) => {
		if ($tok.len() >= $i + 1) && ($tok[$i].len() > 0) {
			Some($tok[$i])
		} else { 
			None
		}
	}
}

pub fn show(mut cpu : CPU, mut system : Arc<RefCell<GBSystem>>) {

	let (mut cpu, mut system) = (cpu, &mut system);
	let mut breakpoints : Vec<u16> = Vec::new();

	println!("Welcome to rustyboy.");
	println!("Type \"help\" for help, or \"exit\" to exit.");
	loop {
		let line = readline::readline(&CString::new("rb> ").unwrap()).unwrap();
		let cmd = line.to_str().unwrap();	
		let tokens : Vec<&str> = cmd.split(char::is_whitespace).collect();
		
		match tokens[0] {
			"b" | "break" => {
				let addr_str = extract_arg!(tokens, 1, "address literal or clear");
				if addr_str == "clear" {
					println!("deleted all breakpoints");
					breakpoints.clear();
					continue					
				}
				let addr = match u16::from_str_radix(&addr_str, 16) {
					Ok(a) => a,
					Err(e) => {
						println!("Parse error: {}", e);
						continue
					}
				};
				if !breakpoints.contains(&addr) {
					breakpoints.push(addr);
				}
				let bytes = cpu.fetch(addr);
				let insn = cpu.decode(bytes);
				println!("breakpoint at 0x{:>04x}: {}", addr, insn);
			},
			"l" | "list" => { //list next n instructions
				let mut n = if let Some(n_str) = extract_opt_arg!(tokens, 1) {  
					match u16::from_str_radix(&n_str, 10) {
						Ok(n) => n,
						Err(e) => {
							println!("Parse error: {}", e);
							continue
						}
					}
				} else {
					10
				};
				let mut addr = cpu.regs.pc;
				while n > 0 {
					let bytes = cpu.fetch(addr);
					let insn = cpu.decode(bytes);
					println!("{:>04x}: {}", addr, insn);
					addr = addr.wrapping_add(insn.length);
					n-=1;
				}
			},
			"set" => { // set register to value
				let reg_str = extract_arg!(tokens, 1, "register");
				let value_str = extract_arg!(tokens, 2, "value");
				let val = match u16::from_str_radix(&value_str, 16) {
					Ok(n) => n,
					Err(e) => {
						println!("Parse error: {}", e);
						continue
					}
				};
				match reg_str {
					"af" | "bc" | "de" | "hl" | "sp" | "pc"  => cpu.regs.set16(Reg16Operand::from_str(reg_str), val),
					"a" | "b" | "c" | "d" | "e" | "h" | "l" => cpu.regs.set8(Reg8Operand::from_str(reg_str), val as u8),
					_ => println!("Invalid register") 
				}
			}
			"s" | "step" => { //execute single instruction
			}, 
			"load" => {
				
				let filename = extract_arg!(tokens, 1, "filename");

			    let rom = match Rom::create_from_file(filename) {
			        Ok(n) => n,
			        Err(err) => {
			            println!("Error: {}" ,err);
			            continue
			        }
			    };
				*system = Arc::new(RefCell::new(GBSystem::new(rom)));
				cpu.reset();
			},
			"reset" => cpu.reset(),
			"p" | "print" => {
				if let Some(what) = extract_opt_arg!(tokens, 1) {
					//try address first
					match u16::from_str_radix(&what, 16) {
						Ok(addr) => {
							println!("({:>04x}) = {:>02x}", addr, system.borrow_mut().read8(addr));
							continue
						} 
						Err(e) => {}
					}
					//now try regs
					match what {
						"af" | "bc" | "de" | "hl" | "sp" | "pc" => println!("{} = {:>04x}", what, cpu.regs.get16(Reg16Operand::from_str(what))),
						"a" | "b" | "c" | "d" | "e" | "h" | "l" => println!("{} = {:>02x}",what, cpu.regs.get8(Reg8Operand::from_str(what))),
						"f" | "flags" => println!("Z={}, N={}, H={}, C={}", cpu.regs.z_flag() as u8, cpu.regs.n_flag() as u8, cpu.regs.h_flag() as u8, cpu.regs.c_flag() as u8), 
						"ime" => println!("IME = {}", cpu.regs.ime as u8),
						_ => println!("Invalid print operand")
					}
				} else {
					cpu.regs.dump()
				}
			},
			"run" => {
				let max_insns = if let Some(n_str) = extract_opt_arg!(tokens, 1) {  
					match u16::from_str_radix(&n_str, 10) {
						Ok(n) => Some(n),
						Err(e) => {
							println!("Parse error: {}", e);
							continue
						}
					}
				} else {
					None
				};
				
				if let Some(mut n) = max_insns {
					while n > 0 {
						cpu.run_instruction();
						n-=1
					}
				} else {
					loop {
						cpu.run_instruction();

						if breakpoints.contains(&cpu.regs.pc) {
							println!("Breakpoint triggered at {:>04x}", cpu.regs.pc); 
							break;							
						}						
					}
				}
			}, //TODO run emulation. opt arg: num 
			"i" | "info" => {
				system.borrow().mbc.rom.dump_header();
			}, //TODO print ROM info
			"h" | "?" | "help" => {
			}
			"q" | "quit" | "exit" => break,
			"" => {}, //ignore whitespace 
			_ => println!("Invalid command.")

		}
		
	}
}