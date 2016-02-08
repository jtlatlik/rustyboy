use std::fmt;

use super::register::*;
use super::operands::*;
use super::operands::Reg8Operand::*;
use super::operands::Reg16Operand::*;

pub const CARRY_FLAG : i32 = 4;
pub const HALFCARRY_FLAG : i32 = 5;
pub const SUB_FLAG : i32 = 6;
pub const ZERO_FLAG : i32 = 7;

pub struct GBRegisters {
    af : Register,
    pub bc : Register,
    pub de : Register,
    pub hl : Register,
    pub sp : Register,
    pub pc : Register,
    pub ime : bool,
}

impl GBRegisters {
	
	pub fn new() -> GBRegisters {
        GBRegisters {
            af : 0,
            bc : 0,
            de : 0,
            hl : 0,
            sp : 0,
            pc : 0,
            ime : true,
        }
	}
    
    pub fn set8(&mut self, reg: Reg8Operand, val: u8) {
        match reg {
            a => self.af.set_high(val),
            b => self.bc.set_high(val),
            c => self.bc.set_low(val),
            d => self.de.set_high(val),
            e => self.de.set_low(val),
            h => self.hl.set_high(val),
            l => self.hl.set_low(val),
        }
    }
    
	pub fn set16(&mut self, reg : Reg16Operand, val: u16) {
        match reg {
            af => self.af = val & 0xfff0, //lowest nibble always zero
            bc => self.bc = val,
            de => self.de = val,
            hl => self.hl = val,
            sp => self.sp = val,
            pc => self.pc = val,
        }
	}

    pub fn get16(&self, reg : Reg16Operand) -> u16 {
        match reg {
            af => self.af,
            bc => self.bc,
            de => self.de,
            hl => self.hl,
            sp => self.sp,
            pc => self.pc,
        }
    }

    pub fn get8(&self, reg : Reg8Operand) -> u8 {
        match reg {
            a => self.af.high(),
            b => self.bc.high(),
            c => self.bc.low(),
            d => self.de.high(),
            e => self.de.low(),
            h => self.hl.high(),
            l => self.hl.low(),
        }
    }
    
    pub fn set_flag(&mut self, flag: i32, v: bool) {
    	if v {
    		self.af |= 1<< flag    		
    	} else {
    		self.af &= !(1<<flag)
    	}
    }

    pub fn z_flag(&self) -> bool {
        (self.af & (1<<ZERO_FLAG)) != 0
    }
    
    pub fn c_flag(&self) -> bool {
        (self.af & (1<<CARRY_FLAG)) != 0
    }
    
    pub fn n_flag(&self) -> bool {
        (self.af & (1<<SUB_FLAG)) != 0
    }
    
    pub fn h_flag(&self) -> bool {
        (self.af & (1<<HALFCARRY_FLAG)) != 0
    }
    
    pub fn cc_satisfied(&self, cond : CCOperand) -> bool {
        match cond {
            CCOperand::z => self.z_flag(),
            CCOperand::nz => !self.z_flag(),
            CCOperand::c => self.c_flag(),
            CCOperand::nc => !self.c_flag(),
            _ => true
        }
    }
    
    pub fn dump(&self) {
        println!("{}", self);
    }
}

impl fmt::Display for GBRegisters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, concat!("AF={:>02x}|{:>02x}  BC={:>02x}|{:>02x}  DE={:>02x}|{:>02x}  HL={:>02x}|{:>02x} ",
                 "SP={:>04x} PC={:>04x} | Z={} N={} H={} C={} |"), 
                 self.af.high(), self.af.low(), self.bc.high(), self.bc.low(),
                 self.de.high(), self.de.low(), self.hl.high(), self.hl.low(),
                 self.sp, self.pc,
                 self.z_flag() as usize, self.n_flag()  as usize, self.h_flag() as usize,self.c_flag()  as usize)
    }
}