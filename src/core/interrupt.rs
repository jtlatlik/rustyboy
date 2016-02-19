use super::cpu::CPU;
use system::system::GBSystem;

impl CPU {
	
	pub fn handle_interrupts(&mut self) -> Option<u32> {
		//only handle interrupts when interrupts are enabled
		if self.regs.ime || self.halt_mode {
			let (iflags,  ienable);
			{
				let sys = self.sys.borrow();
				let iregs = sys.interrupt_regs.borrow_mut();
				iflags = *iregs.iflags;
				ienable = *iregs.ienable;
			}

			for i in 0..5 { //check from highest to lowest priority
				
				if iflags & ienable & (1<<i) != 0 {
					
					if !self.regs.ime {
						//only possibility to reach this is through halt mode
						self.halt_mode = false;
						let next_pc = self.regs.pc.wrapping_add(1); //length of halt
						self.regs.pc = next_pc;
						//DMG bug: if IME==0 and halt_mode=true then the next byte after halt is executed twice...
						let mut insn_bytes = self.fetch(next_pc);
						insn_bytes[2] = insn_bytes[1];
						insn_bytes[1] = insn_bytes[0];
						let insn = self.decode(insn_bytes);
						let cycles = self.execute(insn).unwrap();
						
						return Some(cycles);
					} else {
						let mut sys = self.sys.borrow_mut();
						
						//reset interrupt flag
						sys.interrupt_regs.borrow_mut().iflags.data &= !(1<<i);

						let isr_addr = match i {
							0 => 0x40,
							1 => 0x48,
							2 => 0x50,
							3 => 0x58,
							4 => 0x60,
							_ => unreachable!()
						};					
						//disable interrupts
						self.regs.ime = false;
						//push pc to stack
						self.regs.sp -= 2;
						sys.write16(self.regs.sp, self.regs.pc);
						self.regs.pc = isr_addr;
						
						return Some(20);
					}					
				}
			}
		}
		None
	}
}