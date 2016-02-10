use std::ops::{Deref, DerefMut};
use std::fmt;
use super::system::MemoryAccess;

pub struct IORegister {
	pub data : u8,
	wmask : u8,
	rmask : u8
}


impl Deref for IORegister {
	type Target = u8;
	fn deref(&self) -> &u8 {
		&self.data
	}
}

impl DerefMut for IORegister {
	fn deref_mut(&mut self) -> &mut u8 {
		&mut self.data
	}
}


impl IORegister {
	
	pub fn new() -> IORegister {
		IORegister {
			data : 0,
			wmask : 0xff,
			rmask : 0xff
		}
	}
	
	pub fn set(mut self, data : u8) -> IORegister {
		self.data = data;
		self
	}
	
	pub fn write_mask(mut self, mask : u8) -> IORegister {
		self.wmask = mask;
		self
	}

	pub fn read_mask(mut self, mask : u8) -> IORegister {
		self.rmask = mask;
		self
	}
	
	pub fn write_only(mut self) ->  IORegister {
		self.rmask = 0x00;
		self
	}
	
	pub fn read_only(mut self) ->  IORegister {
		self.wmask = 0x00;
		self
	}
}

impl Default for IORegister {
	fn default() -> IORegister {
		IORegister::new()
	}	
}


impl fmt::Display for IORegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl MemoryAccess for IORegister {
	
	fn write(&mut self, _: u16, data: u8) {
		self.data = (data & self.wmask) | ((!self.wmask) & self.data)
	}
	
	fn read(&mut self, _: u16) -> u8 {
		self.data & self.rmask
	}
}