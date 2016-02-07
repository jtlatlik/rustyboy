#![allow(dead_code,non_camel_case_types)]
mod core;
mod rom;
mod system;
mod gui;

use std::env;
use std::process;

use core::cpu::CPU;
use system::system::GBSystem;

use std::str::FromStr;

use rom::*;

use std::sync::{Arc, RwLock};
use std::rc::Rc;

//use core::memory::*;
//use core::register::Contents;

fn main() {
    let args : Vec<_> = env::args().collect();
    let mut max_insns = None;
    match args.len() {
    	2 => {}, 
    	3 => {
    		max_insns = Some(u64::from_str(&args[2]).unwrap())
    	},
    	_ => {
			println!("Usage: {} ROM_FILE.gb", args[0]);
        	process::exit(1)
    	}
    }
    
    let filename = &args[1];
    let rom = match Rom::create_from_file(filename) {
        Ok(n) => n,
        Err(err) => {
            println!("Error: {}" ,err);
            process::exit(1)
        }
    };
    
    //rom.dump_header();
    
    


	//create CPU peripherals
	let sys = Arc::new(RwLock::new(GBSystem::new(rom)));
	//create CPU
	let mut cpu = CPU::new(sys.clone());
	
	cpu.regs.pc = 0x100;
    cpu.regs.af = 0x01b0;
    cpu.regs.bc = 0x0013;
    cpu.regs.de = 0x00d8;
    cpu.regs.hl = 0x014d;
    cpu.regs.sp = 0xfffe;
	
	//create CPU
	let gui_handle = gui::init(sys.clone());
	let sys_handle = system::start(cpu, sys.clone());

	match gui_handle.join() {
		Ok(_) => (),
		Err(_) => panic!("gui thread panic")
	};
	
}