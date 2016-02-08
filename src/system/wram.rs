use super::system::MemoryAccess;

pub const WRAM_BANK_SIZE :usize = 4*1024; //4K per WRAM bank

pub struct WRAMBank(pub Box<[u8; WRAM_BANK_SIZE]>);

pub struct ZeroPageRAM(pub Box<[u8; 128]>);


impl MemoryAccess for WRAMBank {
	fn read(&mut self, addr: u16) -> u8 {
		self.0[addr as usize]
	}
	
	fn write(&mut self, addr: u16, data: u8) {
		self.0[addr as usize] = data;
	}
}

impl MemoryAccess for ZeroPageRAM {
	fn read(&mut self, addr: u16) -> u8 {
		self.0[addr as usize]
	}
	
	fn write(&mut self, addr: u16, data: u8) {
		self.0[addr as usize] = data;
	}
}