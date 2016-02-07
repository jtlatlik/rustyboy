pub type Register = u16;

pub trait Contents {
    fn low(&self) -> u8;
    fn high(&self) -> u8;
    
    fn set_low(&mut self, value : u8);
    fn set_high(&mut self, value : u8);
}

    
impl Contents for Register {
    
    fn high(&self) -> u8 {
        (*self >> 8) as u8
    }

    fn low(&self) -> u8 {
        *self as u8
    }
    
    fn set_low(&mut self, value : u8) {
        *self = (*self & 0xff00) | (value as u16);
    }
    fn set_high(&mut self, value : u8) {
        *self = (*self & 0x00ff) | ((value as u16) << 8);
    }
}