use super::cpu::CPU;

impl CPU {
	
	pub fn handle_interrupts(&mut self) -> Option<u32> {
		//only handle interrupts when interrupts are enabled
		if self.regs.ime {
			
			let mut lock = self.mem.write().unwrap();
			let sys = &mut lock;
			
			let (iflags, ienable) = (sys.interrupt_regs.iflags.data, sys.interrupt_regs.ienable.data);
			
			for i in 0..5 { //check from highest to lowest priority
				
				if iflags & ienable & (1<<i) != 0 {
					
					//reset interrupt flag
					sys.interrupt_regs.iflags.data &= !(1<<i);
					
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
					
					//enter ISR
					self.regs.pc = isr_addr;
					print!("Interrupt: 0x{:>02x}", isr_addr);
					return Some(5);
				}
			}
			
		}
		None
	}
}