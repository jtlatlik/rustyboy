pub mod system;

pub mod video;
mod ioregister;
mod sound;
mod timer;
mod mbc;
mod interrupt;
mod wram;
mod serial;

use std::sync::Arc;
use std::cell::RefCell;

use rom::Rom;
use core::cpu::CPU;
use self::system::*;

pub fn init(rom: Rom) -> (CPU, Arc<RefCell<GBSystem>>) {

	//create CPU peripherals
	let raw_sys = GBSystem::new(rom);
	let sys = Arc::new(RefCell::new(raw_sys));
	
	//create CPU
	let cpu = CPU::new(sys.clone());

    (cpu, sys)
}