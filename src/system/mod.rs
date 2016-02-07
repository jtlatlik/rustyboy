pub mod system;

pub mod video;
mod sound;
mod timer;

use std::io::{self, Read};

use std::sync::{Arc, RwLock};
use std::thread::*;
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;

use core::cpu::CPU;
use self::system::*;


pub type ThreadSafeSystem = Arc<RwLock<GBSystem>>;

pub fn start(mut cpu : CPU, sys: ThreadSafeSystem) -> JoinHandle<()> {

	thread::Builder::new().name("cpu_system".to_string()).spawn(move || {
		loop  {
			{
				
				let mut cycles = 0;
				
				//handle interrupts
				if let Some(c) = cpu.handle_interrupts() {
					cycles = c;
				} else {
				
					//print register contents
					print!("{} ", cpu.regs);
					
					//fetch instruction
					let mut insn_bytes = [0;3];
					let pc = cpu.regs.pc;
					{
						let mut lock2  = sys.write().unwrap();
						let mem = &mut lock2;
						insn_bytes = [mem.read8(pc), mem.read8(pc+1), mem.read8(pc+2)];
					}
					
					//decode insn
					let insn = cpu.decode(insn_bytes);
	
					//print raw and decoded instruction
					print!("{:04x}:", pc);
					for i in 0..3 {
						if i < insn.length {
							print!(" {:>02x}", insn_bytes[i as usize])
						} else {
							print!("   ")
						}
						
					}
					print!(" {}\t", insn);
					
					//execute instruction
					cycles = match cpu.execute(insn) {
						Ok(c) => c,
						Err(e) => {
							panic!("\n Fatal error during execution: {:?}", e);
						}
					};
				}
				
				//update periphery
				{
					let mut lock2  = sys.write().unwrap();
					let sys = &mut lock2;
					sys.update(cycles);
				}
				
				
				println!("");
				//let mut str ="".to_string();
				//let input = io::stdin().read_line(&mut str).ok().expect("Failed to read line");
			}
							
		};			
	}).unwrap()
}