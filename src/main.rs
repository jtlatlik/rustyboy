#![allow(dead_code,non_camel_case_types)]
mod core;
mod rom;
mod system;
mod gui;
mod prompt;
mod logger;

extern crate getopts;
#[macro_use] extern crate log;
extern crate time;

#[macro_use]
extern crate bitflags;


use std::env;
use std::process;
use std::io::Write;
use std::fs::File;
use getopts::Options;
use core::instruction::{Instruction, InstructionType};
use core::operands::{Reg16Operand,Operand,CCOperand};
use rom::*;


fn main() {
    let args : Vec<_> = env::args().collect();
    
    let mut opts = Options::new();
    
    opts.optflag("i", "interactive", "start in interactive mode");
    opts.optflag("h", "help", "print this help information");
    opts.optflag("l", "log", "enable logging (disabled by default)");
    opts.optopt("t", "trace", "set trace output file name", "FILE");
    
    let progname = args[0].clone();
    
    let matches = match opts.parse(&args[1..]) {
    	Ok(m) => m,
    	Err(e) => {
    		writeln!(&mut std::io::stderr(), "Error: {}", e.to_string()).unwrap();
    		return
    	}
    };
    
    if matches.opt_present("h") {
    	print_usage(opts, &progname);
    	return;
    }
    
    if matches.opt_present("l") {
    	logger::init().unwrap();
    }
    
	//positional arguments
	if matches.free.len() != 1 {
    	print_usage(opts, &progname);
    	return
	}
	
	let romfile = matches.free[0].clone();

    let rom = match Rom::create_from_file(&romfile) {
        Ok(n) => n,
        Err(err) => {
            println!("Error: {}" ,err);
            process::exit(1)
        }
    };
    
    let (mut cpu, sys) = system::init(rom);
	    
    if let Some(filename) = matches.opt_str("t") {
    	cpu.set_trace_file(File::create(filename).unwrap())
    }
	
	let mut gui = gui::init();
	
    if matches.opt_present("i") {
    	prompt::show(cpu, sys, gui);
    	return
    }

	let mut real_time : f64 = 0.0;
	let mut emulation_time :f64 = 0.0;
	let mut t0 = time::precise_time_ns();
	loop {
		emulation_time += cpu.run_instruction();
		gui.update(&mut cpu);
		let t1 = time::precise_time_ns();
		real_time += (t1 - t0) as f64;
		t0 = t1;
		if gui.speed_mode {
			real_time = emulation_time;
		}
		let diff = emulation_time - real_time;
		if diff > 1000000.0 && !gui.speed_mode { //1ms
			std::thread::sleep(std::time::Duration::new(0, diff as u32))
		}
	}	
}

fn print_usage(opts : Options, progname : &str) {
	let brief = format!("Usage: {} ROM_FILE.gb [options]", progname);
	print!("{}", opts.usage(&brief));
}

//	let mut crc : u32 = !0;
//	for flags in 0..16 {
//		for acc in 0..256 {
//			let insn = Instruction {
//				itype : InstructionType::daa,
//				dest : Operand::none,
//				src : [Operand::none; 2],
//				cc : CCOperand::none,
//				length : 1
//			};
//			let pre = (acc << 8) | (flags << 4);
//			cpu.regs.set16(Reg16Operand::af, pre);
//			cpu.execute(insn);
//			let post = cpu.regs.get16(Reg16Operand::af);
//			//calculate crc checksums
//			let crcacc = update_crc(crc ^ (((post >> 8) as u32)&0xff));
//			let crcflags = update_crc(crcacc ^ ((post as u32 )&0xff));
//			println!("{:04x} {:04x}\t{:08x} {:08x}", pre, post, crcacc, crcflags);
//			crc =crcflags;
//		}
//		
//	}
//	return;