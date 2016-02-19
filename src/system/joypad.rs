
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
	btn_keys : ButtonKeyMask
}

impl Joypad {
	
	pub fn new() -> Joypad {
		Joypad {
			sel_mask : SelectMask::from_bits_truncate(0),
			dir_keys : DirKeyMask::from_bits_truncate(0b1111),
			btn_keys : ButtonKeyMask::from_bits_truncate(0b1111)
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
	
	pub fn set_register(&mut self, data :u8) {
		self.sel_mask = SelectMask::from_bits_truncate(data)
	}

	pub fn set_start_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.btn_keys.insert(KEY_START)
		} else { 
			self.btn_keys.remove(KEY_START)
		}
	}
	
	pub fn set_select_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.btn_keys.insert(KEY_SELECT)
		} else { 
			self.btn_keys.remove(KEY_SELECT)
		}			
	}
	
	pub fn set_a_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.btn_keys.insert(KEY_A)
		} else { 
			self.btn_keys.remove(KEY_A)
		}
	}
	
	pub fn set_b_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.btn_keys.insert(KEY_B)
		} else { 
			self.btn_keys.remove(KEY_B)
		}
	}
	
	pub fn set_up_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.dir_keys.insert(KEY_UP)
		} else { 
			self.dir_keys.remove(KEY_UP)
		}
	}
	
	pub fn set_down_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.dir_keys.insert(KEY_DOWN)
		} else { 
			self.dir_keys.remove(KEY_DOWN)
		}
	}
	
	pub fn set_left_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.dir_keys.insert(KEY_LEFT)
		} else { 
			self.dir_keys.remove(KEY_LEFT)
		}
	}
	
	pub fn set_right_pressed(&mut self, pressed: bool) {
		if !pressed {
			self.dir_keys.insert(KEY_RIGHT)
		} else { 
			self.dir_keys.remove(KEY_RIGHT)
		}
	}	
}