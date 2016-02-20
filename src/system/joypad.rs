use std::rc::Rc;
use std::cell::RefCell;
use super::interrupt::InterruptRegisters;

bitflags! {
    flags SelectMask: u8 {
        const BUTTON_KEYS = 1<<4,
        const DIR_KEYS    = 1<<5,
    }
}

bitflags! {
    flags DirKeyMask: u8 {
        const KEY_RIGHT = 1<<0,
        const KEY_LEFT  = 1<<1,
        const KEY_UP    = 1<<2,
        const KEY_DOWN  = 1<<3
    }
}

bitflags! {
    flags ButtonKeyMask: u8 {
        const KEY_A      = 1<<0,
        const KEY_B      = 1<<1,
        const KEY_SELECT = 1<<2,
        const KEY_START  = 1<<3
    }
}

pub struct Joypad {
	
	sel_mask : SelectMask,
	dir_keys : DirKeyMask,
	btn_keys : ButtonKeyMask,
	interrupt_regs : Rc<RefCell<InterruptRegisters>>
}

impl Joypad {
	
	pub fn new(iregs : Rc<RefCell<InterruptRegisters>>) -> Joypad {
		Joypad {
			sel_mask : SelectMask::from_bits_truncate(0),
			dir_keys : DirKeyMask::from_bits_truncate(0b1111),
			btn_keys : ButtonKeyMask::from_bits_truncate(0b1111),
			interrupt_regs : iregs
		}
	}

	pub fn get_register(&self) -> u8 {
		
		let value = self.sel_mask.bits();

		if self.sel_mask.contains(BUTTON_KEYS) {
			value | self.btn_keys.bits() 
		} else if self.sel_mask.contains(DIR_KEYS) {
			value | self.dir_keys.bits()
		} else {
			value | 0xf
		}
	}
	
	fn req_interrupt(&self) {
		let mut iregs= self.interrupt_regs.borrow_mut();
		*iregs.iflags |= 1<<4;
	}
	
	pub fn set_register(&mut self, data :u8) {
		self.sel_mask = SelectMask::from_bits_truncate(data)
	}

	pub fn set_start_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.btn_keys.insert(KEY_START);
			self.req_interrupt()
			
		} else { 
			self.btn_keys.remove(KEY_START)
		}
	}
	
	pub fn set_select_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.btn_keys.insert(KEY_SELECT);
			self.req_interrupt()
		} else { 
			self.btn_keys.remove(KEY_SELECT)
		}			
	}
	
	pub fn set_a_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.btn_keys.insert(KEY_A);
			self.req_interrupt()
		} else { 
			self.btn_keys.remove(KEY_A)
		}
	}
	
	pub fn set_b_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.btn_keys.insert(KEY_B);
			self.req_interrupt()
		} else { 
			self.btn_keys.remove(KEY_B)
		}
	}
	
	pub fn set_up_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.dir_keys.insert(KEY_UP);
			self.req_interrupt()
		} else { 
			self.dir_keys.remove(KEY_UP)
		}
	}
	
	pub fn set_down_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.dir_keys.insert(KEY_DOWN);
			self.req_interrupt()
		} else { 
			self.dir_keys.remove(KEY_DOWN)
		}
	}
	
	pub fn set_left_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.dir_keys.insert(KEY_LEFT);
			self.req_interrupt()
		} else { 
			self.dir_keys.remove(KEY_LEFT)
		}
	}
	
	pub fn set_right_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.dir_keys.insert(KEY_RIGHT);
			self.req_interrupt()
		} else { 
			self.dir_keys.remove(KEY_RIGHT)
		}
	}	
}