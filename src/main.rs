#![allow(dead_code,non_camel_case_types)]
mod core;
mod rom;
mod system;
mod gui;
mod prompt;
mod logger;

extern crate getopts;
#[macro_use] extern crate log;

use std::env;
use std::process;
use std::io::Write;

use std::fs::File;

use getopts::Options;

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
    
    let mut trace_handle = None;
    
    if let Some(filename) = matches.opt_str("t") {
    	//open file...
    	cpu.trace_enabled = true;
    	trace_handle = Some(File::create(filename).unwrap());
    }
	
    if matches.opt_present("i") {
    	prompt::show(cpu, sys);
    	return
    }
	
	gui::init(&sys.borrow());
	
	loop {
		let trace = cpu.run_instruction();
		
		if let Some(trace_line) = trace {

			if let Some(ref mut tracefile) = trace_handle {
				tracefile.write_all(trace_line.as_bytes()).unwrap();
			}
		}
	}
	
}

fn print_usage(opts : Options, progname : &str) {
	let brief = format!("Usage: {} ROM_FILE.gb [options]", progname);
	print!("{}", opts.usage(&brief));
}