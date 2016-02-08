use super::system::MemoryAccess;
use super::ioregister::IORegister;

macro_rules! bits {
	( $($bit:expr)* ) => ( 0x00 $( | (1<<$bit) )* )
}

pub struct DividerRegister(pub u8);

pub struct TimerRegisters {
	pub divider : DividerRegister,
	pub counter : IORegister,
	pub modulo : IORegister,
	pub control : IORegister
}

impl TimerRegisters {
	pub fn update(delta : u32) {
		unimplemented!()
	}
}

impl Default for TimerRegisters {
	fn default() -> TimerRegisters {
		
		TimerRegisters {
			divider : DividerRegister(0),
			counter : IORegister::new(),
			modulo : IORegister::new(),
			control : IORegister::new().write_mask(bits!(2 1 0))
		}
	}
}

impl MemoryAccess for DividerRegister {
	
	fn write(&mut self, _: u16, _: u8) {
		self.0 = 0
	}
	
	fn read(&mut self, _: u16) -> u8 {
		self.0
	}
}