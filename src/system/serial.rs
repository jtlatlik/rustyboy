use std::ops::{Deref, DerefMut};
use super::system::MemoryAccess;
use super::ioregister::IORegister;

macro_rules! bits {
	( $($bit:expr)* ) => ( 0x00 $( | (1<<$bit) )* )
}

pub struct SerialRegisters {
	pub data : IORegister,
	pub control : IORegister	
}

impl Default for SerialRegisters {
	fn default() -> SerialRegisters {
		SerialRegisters {
			control : IORegister::new().write_mask(bits!(7 1 0)),
			data : IORegister::new()
		}
	}
}