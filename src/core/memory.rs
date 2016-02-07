pub trait Memory {
	
    fn read8(&mut self, addr: u16) -> u8;
    
    fn write8(&mut self, addr: u16, data: u8);
    
    fn read16(&mut self, addr: u16) -> u16 {
    	((self.read8(addr+1) as u16) << 8) | (self.read8(addr) as u16)
    }
    
    fn write16(&mut self, addr: u16, data: u16) {
    	self.write8(addr, data as u8);
    	self.write8(addr+1, (data >> 8) as u8)
    }
    
}

pub struct MemoryDummy;
    
impl Memory for MemoryDummy {
    fn read8(&mut self, addr: u16) -> u8 {
        0
    }

    fn write8(&mut self, addr: u16, data: u8) {
        
    }
}