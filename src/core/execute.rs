use super::cpu::CPU;
use super::gb::*;
use super::memory::*;
use super::instruction::*;
use super::instruction::InstructionType::*;
use super::operands::*;
use super::operands::Operand::*;
use super::register::Contents;
use self::ExecuteError::*;

#[derive(Debug)]
enum ExecuteError {
	InvalidDestOperand(Operand),
	InvalidSrcOperand(Operand),
	InvalidInstruction
}


macro_rules! try_r8_i8_mem8 {
	($src:expr,$regs:ident, $mem:ident, $cycles:ident) => ( try!( match $src {
		reg8(rs) => {
			Ok($regs.get8(rs))
		},
		imm8(i) => {
			$cycles += 4;
			Ok(i as u8)
		}
		mem_reg(ra) => {
			$cycles += 4;
			Ok($mem.read8($regs.get16(ra)))
		},
		mem_imm(a) => {
			$cycles += 12;
			Ok($mem.read8(a))
		}
		mem_io_imm(o) => {
			$cycles += 8;
			Ok($mem.read8(0xff00 + (o as u16)))
		},
		mem_io_reg(Reg8Operand::c) => {
			$cycles += 4;
			Ok($mem.read8(0xff00 + ($regs.get8(Reg8Operand::c) as u16)))
		}
        _ => Err(ExecuteError::InvalidSrcOperand($src))
	}))
}

macro_rules! try_r8_mem8 {
	($src:expr,$regs:ident, $mem:ident, $cycles:ident) => ( try!( match $src {
		reg8(rs) => {
			Ok($regs.get8(rs))
		},
		mem_reg(ra) => {
			$cycles += 4;
			Ok($mem.read8($regs.get16(ra)))
		},
        _ => Err(ExecuteError::InvalidSrcOperand($src))
	}))
}

macro_rules! try_r8_i8 {
	($src:expr,$regs:ident, $cycles:ident) => ( try!( match $src {
		reg8(rs) => {
			Ok($regs.get8(rs))
		},
		imm8(i) => {
			$cycles += 4;
			Ok(i as u8)
		},
        _ => Err(ExecuteError::InvalidSrcOperand($src))
	}))
}

macro_rules! try_i8 {
	($src:expr, $cycles:ident) => ( try!( match $src {
		imm8(i) => {
			$cycles += 4;
			Ok(i as u8)
		},
        _ => Err(ExecuteError::InvalidSrcOperand($src))
	}))
}


macro_rules! try_r8 {
	($src:expr,$regs:ident) => ( try!( match $src {
		reg8(rs) => {
			Ok($regs.get8(rs))
		},
        _ => Err(ExecuteError::InvalidSrcOperand($src))
	}))
}

macro_rules! try_r16_i16 {
	($src:expr,$regs:ident, $mem:ident, $cycles:ident) => ( try!( match $src {
		reg16(rs) => {
			$cycles += 4;
			Ok($regs.get16(rs))
		},
		imm16(i) => {
			$cycles += 8;
			Ok(i as u16)
		},
        _ => Err(ExecuteError::InvalidSrcOperand($src))
	}))
}

macro_rules! try_r16 {
	($src:expr, $errtype:ident ) => ( try!( match $src {
		reg16(rs) => Ok(rs),
        _ => Err(ExecuteError::$errtype($src))
	}))
}

//impl<M: Memory> CPU<M> {
impl CPU {
	
    pub fn execute(&mut self, insn: Instruction) -> Result<u32, ExecuteError> {
        
        let regs = &mut self.regs;
        let mut mem = self.sys.borrow_mut();
        
        let mut cycles = 4;
        let mut next_pc = regs.pc.wrapping_add(insn.length as u16);
        
        cycles = try!(match insn.itype {
            //Loadcommands
            ld => {
            	match insn.dest {
            		reg8(rd) => { //8-bit load or move
            			let data = try_r8_i8_mem8!(insn.src[0], regs, mem, cycles);
            			regs.set8(rd, data);
            			Ok(cycles)
            		},
            		reg16(rd) => { //16-bit move
            			if let imm8(off) = insn.src[1] { //ld hl, sp+r8 
            				let acc_in = regs.sp;
            				let op = off as i8;
                     		let (_, _,_,h,c) = arith_op(add, acc_in as u8, op as u8, 0);
                        	let result = acc_in.wrapping_add(op as u16);
							regs.set_flag(ZERO_FLAG, false);
							regs.set_flag(SUB_FLAG, false);
                        	regs.set_flag(HALFCARRY_FLAG, h);
                        	regs.set_flag(CARRY_FLAG, c);
                        	regs.hl = result;
                        	Ok(12)
            			} else { // ld rd, ...
            				cycles = 4;
            				let data = try_r16_i16!(insn.src[0], regs, mem, cycles);
            				regs.set16(rd, data);
            				Ok(cycles)
            			}
            		},
            		mem_imm(addr) => { //direct store
						match insn.src[0] {
							reg8(r) => {
		            			let data = regs.get8(r);
		            			mem.write8(addr, data);
		            			Ok(16)
							}, 
							reg16(Reg16Operand::sp) => {
								let data =regs.get16(Reg16Operand::sp);
								mem.write16(addr, data);
								Ok(20)
							},
							_ => Err(ExecuteError::InvalidSrcOperand(insn.src[0]))
						}
            		},
            		mem_reg(ra) => { //8-bit indirect store
            			cycles = 8;
            			let addr = regs.get16(ra);
            			let data = try_r8_i8!(insn.src[0], regs, cycles);
            			mem.write8(addr, data);
            			Ok(cycles)
            		},
            		mem_io_imm(o) => { //8-bit offset direct store
            			let addr = 0xff00 + (o as u16);
            			let data =  try_r8!(insn.src[0], regs);
            			mem.write8(addr, data);
            			Ok(12)
            		},
            		mem_io_reg(ro) => { //8-bit offset indirect store
            			let addr = 0xff00 + (regs.get8(ro) as u16);
            			let data =  try_r8!(insn.src[0], regs);
            			mem.write8(addr, data);
            			Ok(8)
            		},
            		_ => Err(InvalidDestOperand(insn.dest))          		
            	}
            },
            ldi | ldd => {
            	match insn.dest {
            		reg8(Reg8Operand::a) => {
            			let data = mem.read8(regs.hl);
            			regs.set8(Reg8Operand::a, data);
						regs.hl = if insn.itype == ldi {  regs.hl.wrapping_add(1) } else {  regs.hl.wrapping_sub(1)};
            			Ok(8)
            		},
            		mem_reg(Reg16Operand::hl) => {
            			let data = regs.get8(Reg8Operand::a);
            			mem.write8(regs.hl, data);
						regs.hl = if insn.itype == ldi {  regs.hl.wrapping_add(1) } else {  regs.hl.wrapping_sub(1)};
            			Ok(8)
            		},
            		_ => Err(InvalidDestOperand(insn.dest))
            	}
            },
            push => {
            	let reg = try_r16!(insn.src[0], InvalidSrcOperand);
        		regs.sp = regs.sp.wrapping_sub(2);
            	mem.write16(regs.sp, regs.get16(reg));
            	Ok(16)
            },
            pop => {
            	let reg = try_r16!(insn.dest, InvalidDestOperand);
            	let data = mem.read16(regs.sp);
            	regs.set16(reg, data);
            	regs.sp = regs.sp.wrapping_add(2);
            	Ok(12)
            },
            //Arithmetic/logical Commands
            add | adc | sub | sbc | cp => {
                match insn.dest {
                    reg8(rd) => { //8bit op
                        let acc_in = regs.get8(rd);
                        let c_in = regs.c_flag() as u8;
                        let op = try_r8_i8_mem8!(insn.src[0], regs, mem, cycles);

                        let (acc_out, z,n,h,c) = arith_op(insn.itype, acc_in, op, c_in);
                        if insn.itype != cp {
                        	regs.set8(rd, acc_out);
                        }
                        //write flags back
                        regs.set_flag(ZERO_FLAG, z);
                        regs.set_flag(SUB_FLAG, n);
                        regs.set_flag(HALFCARRY_FLAG, h);
                        regs.set_flag(CARRY_FLAG, c);
                        Ok(cycles)
                    },
                    reg16(Reg16Operand::sp) => { //16bit op
                    	cycles = 12;
                        let acc_in = regs.sp;
                        let op = try_i8!(insn.src[0], cycles) as i8;
                        let (_, _,_,h,c) = arith_op(insn.itype, acc_in as u8, op as u8, 0);
						regs.sp = acc_in.wrapping_add(op as u16);
						regs.set_flag(ZERO_FLAG, false);
						regs.set_flag(SUB_FLAG, false);
                        regs.set_flag(HALFCARRY_FLAG, h);
                        regs.set_flag(CARRY_FLAG, c);
                        Ok(cycles)
                    },
					reg16(Reg16Operand::hl) => {
						let acc_in = regs.hl as u32;
						let op = regs.get16(try_r16!(insn.src[0], InvalidSrcOperand));
						
						let acc_in12 = regs.hl & 0x0fff;
						let op12 = op & 0x0fff;

						let result = acc_in.wrapping_add(op as u32);
						let h = (acc_in12 + op12) & 0x1000 != 0;
						let c = result & 0x10000 != 0;
						
						regs.set_flag(SUB_FLAG, false);
                        regs.set_flag(HALFCARRY_FLAG, h);
                        regs.set_flag(CARRY_FLAG, c);
						regs.hl = result as u16;
						Ok(8)
					} 
                    _ => Err(InvalidDestOperand(insn.dest))
                }
            },
            and | xor | or => {
            	let acc = regs.get8(Reg8Operand::a);
            	let op = try_r8_i8_mem8!(insn.src[0], regs, mem, cycles);
            	let (result, z) = logic_op(insn.itype, acc, op);
            	regs.set8(Reg8Operand::a, result);
                regs.set_flag(ZERO_FLAG, z);
                regs.set_flag(SUB_FLAG, false);
                regs.set_flag(HALFCARRY_FLAG, insn.itype == and);
                regs.set_flag(CARRY_FLAG, false);
            	Ok(cycles)
            }
            inc | dec => {
            	match insn.dest {
            		reg8(_)| mem_reg(_) => {
						let op = try_r8_mem8!(insn.dest, regs, mem, cycles);
		            	let (result, z, n, h, _) = arith_op(insn.itype, op, 1, 0);
		            	
		            	match insn.dest {
		            		reg8(rd) => regs.set8(rd, result),
		            		mem_reg(ra) => {
		            			cycles+=4;
		            			mem.write8(regs.get16(ra), result)
		            		}
		            		_ => unreachable!() 
		            	}
		            	regs.set_flag(ZERO_FLAG, z);
		            	regs.set_flag(SUB_FLAG, n);
		            	regs.set_flag(HALFCARRY_FLAG, h);
		            	Ok(cycles)
            		},
            		reg16(rd) => {
            			let op = regs.get16(rd);
            			let result = if insn.itype == inc { op.wrapping_add(1) } else { op.wrapping_sub(1) };
            			regs.set16(rd, result);
            			Ok(8)
            		},
            		_ => Err(ExecuteError::InvalidDestOperand(insn.dest))
            	}
            },
            daa => {
            	let mut acc = regs.get8(Reg8Operand::a) as u16;
				let mut c = false;
				
				if regs.n_flag() {
					if regs.h_flag() { acc = acc.wrapping_sub(0x06) & 0xff }
					if regs.c_flag() { acc = acc.wrapping_sub(0x60); c = true }
				} else {
					if ((acc & 0x0f) > 9) || regs.h_flag() { acc = acc.wrapping_add(0x06) }
					if (acc > 0x9f) || regs.c_flag() { acc = acc.wrapping_add(0x60); c = true }
				}
				let result = acc as u8;
				
				c |= (acc & 0x100) != 0;
				regs.set8(Reg8Operand::a, result);
				regs.set_flag(ZERO_FLAG, result == 0);
				regs.set_flag(HALFCARRY_FLAG, false);
				regs.set_flag(CARRY_FLAG, c);
            	Ok(4)
            },
            cpl => {
            	let data = regs.get8(Reg8Operand::a);
            	regs.set8(Reg8Operand::a, !data);
            	regs.set_flag(SUB_FLAG, true);
            	regs.set_flag(HALFCARRY_FLAG, true);
            	Ok(4)
            },
            //Rotate and Shift Commands
            rlca | rla | rrca | rra => {
            	let acc = regs.get8(Reg8Operand::a);
            	let (result, c) = rot_shift_op(insn.itype, acc, regs.c_flag() as u8);
            	regs.set_flag(ZERO_FLAG, false);
            	regs.set_flag(SUB_FLAG, false);
            	regs.set_flag(HALFCARRY_FLAG, false);
            	regs.set_flag(CARRY_FLAG, c);
            	regs.set8(Reg8Operand::a, result);
            	Ok(cycles)
            },
            rl | rlc | rr | rrc | sla | sra | srl | swap => {
            	cycles = 8;
            	let acc = try_r8_mem8!(insn.dest, regs, mem, cycles);
            	let (result, c) = rot_shift_op(insn.itype, acc, regs.c_flag() as u8);
            	match insn.dest {
            		reg8(rd) => regs.set8(rd, result),
            		mem_reg(ra) => {
            			cycles += 4;
            			mem.write8(regs.get16(ra), result)
            		},
            		_ => unreachable!() 
            	}
            	regs.set_flag(ZERO_FLAG, result == 0);
            	regs.set_flag(SUB_FLAG, false);
            	regs.set_flag(HALFCARRY_FLAG, false);
            	regs.set_flag(CARRY_FLAG, c);
            	Ok(cycles)
            },
            //Singlebit Operation Commands
            bit => {
            	let bitn = try_i8!(insn.src[0], cycles);
            	let src = try_r8_mem8!(insn.src[1], regs, mem, cycles);
            	let z = src & (1 << bitn) == 0; 
            	regs.set_flag(ZERO_FLAG, z);
            	regs.set_flag(SUB_FLAG, false);
            	regs.set_flag(HALFCARRY_FLAG, true);
            	Ok(cycles)
            },
            set | res => {
            	let bitn = try_i8!(insn.src[0], cycles);
            	match insn.dest {
            		reg8(rd) => {
            			let val = regs.get8(rd);
            			regs.set8(rd, if insn.itype == set { val | (1 << bitn)  } else { val & !(1 << bitn) });
            			Ok(cycles)
            		},
            		mem_reg(ra) =>{
            			let addr = regs.get16(ra);
            			let val = mem.read8(addr);
            			mem.write8(addr, if insn.itype == set { val | (1 << bitn)  } else { val & !(1 << bitn) });
            			Ok(16)
            		},
            		_ => Err(ExecuteError::InvalidDestOperand(insn.dest))
            	}
            },
            //CPU-Controlcommands
            ccf => {
            	let v = regs.c_flag();
            	regs.set_flag(SUB_FLAG, false);
            	regs.set_flag(HALFCARRY_FLAG, false);
            	regs.set_flag(CARRY_FLAG, !v);
            	Ok(4)
            },
            scf => {
            	regs.set_flag(SUB_FLAG, false);
            	regs.set_flag(HALFCARRY_FLAG, false);
            	regs.set_flag(CARRY_FLAG, true);
            	Ok(4)
            },
            nop => Ok(4),
            halt => {
            	//println!("entering halt mode");
            	self.halt_mode = true;
            	Ok(4)
            },
            stop => {
            	//println!("entering stop mode");
            	self.stop_mode = true;
            	Ok(4)
            },
            di => {
            	regs.ime = false;
            	Ok(4)
            },
            ei => {
            	regs.ime = true;
            	Ok(4)
            },
            //branches
            jp => {
                if regs.cc_satisfied(insn.cc) {
                    next_pc = try!(match insn.src[0] {
                    	imm16(addr) => {
                    		cycles = 16;
                    		Ok(addr)
                    	},
                    	reg16(Reg16Operand::hl) => Ok(regs.hl),
                    	_ =>  Err(ExecuteError::InvalidSrcOperand(insn.src[0]))
                	});
                    Ok(cycles)
                } else {
                    Ok(12)
                }
            },
            jr => {
                let off8 = try!(match insn.src[0] {
                    imm8(off) => Ok(off as i8),
                    _ =>  Err(ExecuteError::InvalidSrcOperand(insn.src[0]))
                });

                if regs.cc_satisfied(insn.cc) {
					next_pc = next_pc.wrapping_add(off8 as u16);
                	Ok(12)
                } else {
                	Ok(8)
                }
            },
            call => {
            	if regs.cc_satisfied(insn.cc) {
            		regs.sp = regs.sp.wrapping_sub(2);
					mem.write16(regs.sp, next_pc);
					next_pc = try!(match insn.src[0] {
                    	imm16(addr) => Ok(addr),
                    	_ =>  Err(ExecuteError::InvalidSrcOperand(insn.src[0]))
                	});
            		Ok(24)
            	} else {
            		Ok(12)
            	}
            },
            ret | reti=> {            	
            	if regs.cc_satisfied(insn.cc) {
            		cycles = 16;
            		next_pc = mem.read16(regs.sp);
					regs.sp = regs.sp.wrapping_add(2);
            		
            		if insn.cc != CCOperand::none {
            			cycles += 4
            		}
            		
            		if insn.itype == reti {
            			regs.ime = true
            		}
            		
            		Ok(cycles)
            	} else {
            		Ok(8)
            	}
            },
            rst => {
        		regs.sp = regs.sp.wrapping_sub(2);
				mem.write16(regs.sp, next_pc);
				
				next_pc = try!(match insn.src[0] {
                	imm8(addr) => Ok(addr as u16),
                	_ =>  Err(ExecuteError::InvalidSrcOperand(insn.src[0]))
            	});
        		Ok(16)
            },
            invalid => Ok(4)/*Err(InvalidInstruction)*/
        });
        
        //update program counter
        regs.pc = next_pc;
        
        //return cycles 
        Ok(cycles)
    }
}

fn logic_op(itype : InstructionType, acc: u8, op: u8) -> (u8, bool) {
	let result = match itype {
		and => acc & op,
		or => acc | op,
		xor => acc ^ op,
		_ => unreachable!()
	};
	(result, result == 0)
}

fn rot_shift_op(itype: InstructionType, acc: u8, ci: u8) -> (u8, bool) {
	
	let high_c = 0x80 & acc != 0;
	let low_c = acc & 1 != 0;
	match itype {
        rl | rla => ( (acc << 1) | ci , high_c),
        rlc | rlca => (acc.rotate_left(1), high_c),
        rr | rra => ( (acc >> 1) | ( ci << 7), low_c),
        rrc | rrca => (acc.rotate_right(1), low_c),
        sla => ( acc << 1, high_c),
        sra => ( ((acc as i8) >> 1) as u8, low_c),
        srl => ( acc >> 1, low_c),
        swap => ( ((acc & 0xf) << 4) | (acc >> 4), false),
        _ => unreachable!()
	}
}
                                                                 //res, z,  n,    h,    c
fn arith_op(itype : InstructionType, acc: u8, op: u8, ci : u8) -> (u8, bool, bool, bool, bool) {
		
	let ci = match itype {
		adc | sbc => ci,
		_ => 0,
	};
	
	let (acc8, acc4) = ((acc as u16), (acc & 0xf));
	let (op8, op4) = ((op as u16), (op & 0xf));
	let (ci8, ci4) = ((ci as u16), (ci & 0xf));
	
	let (res8, res4, n) = match itype {
		add | adc | inc => {
			(
				acc8.wrapping_add(op8).wrapping_add(ci8),
				acc4.wrapping_add(op4).wrapping_add(ci4),
				false 
			)
		},
		sub | sbc | dec | cp => {
			(
				acc8.wrapping_sub(op8).wrapping_sub(ci8),
				acc4.wrapping_sub(op4).wrapping_sub(ci4),
				true 
			)
		},
		_ => unreachable!()
	};
	let result = res8 as u8;
	let h = res4 >> 4;
	let c = res8 >> 8;
	
	(result, result==0, n, h != 0, c != 0)
}