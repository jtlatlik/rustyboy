use std::sync::{Arc,RwLock};

use super::gb::GBRegisters;
use super::memory::*;
use system::system::GBSystem;

pub struct CPU {

    pub regs : GBRegisters,
    pub mem : Arc<RwLock<GBSystem>>
}

impl CPU {
	
	pub fn new(sys : Arc<RwLock<GBSystem>>) -> CPU {
	    CPU {
	        regs : GBRegisters { ..Default::default() },
	        mem : sys
	    }
	}
}
