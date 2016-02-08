use super::system::MemoryAccess;
use super::ioregister::IORegister;

#[derive(Default)]
pub struct InterruptRegisters {
	pub ienable : IORegister,
	pub iflags : IORegister
}