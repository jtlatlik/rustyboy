use std::fmt;

use self::Operand::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Reg8Operand {
    a,
    b,
    c,
    d,
    e,
    h,
    l
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Reg16Operand {
    af,
    bc,
    de,
    hl,
    sp,
    pc
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CCOperand {
    none,
    z,
    nz,
    c,
    nc
}

#[derive(Debug,PartialEq, Copy, Clone)]
pub enum Operand {
    none,
    imm8(u8),
    imm16(u16),
    reg8(Reg8Operand),
    reg16(Reg16Operand),
    mem_imm(u16),
    mem_reg(Reg16Operand),
    mem_io_imm(u8),
    mem_io_reg(Reg8Operand)
}

impl fmt::Display for Operand {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            mem_imm(a) | imm16(a) => write!(f, "0x{:>04x}", a),
            mem_reg(r) => write!(f, "({:?})", r),
            mem_io_imm(a) =>  write!(f, "(0xff00 + 0x{:>02x})", a),
            mem_io_reg(r) =>  write!(f, "(0xff00 + {:?})", r),
            imm8(i) => write!(f, "0x{:>02x}", i),
            reg8(r) => write!(f, "{:?}", r),
            reg16(r) => write!(f, "{:?}", r),
            none => Ok(())
        }
    }
}

impl fmt::Display for CCOperand {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CCOperand::z => write!(f, "z"),
            CCOperand::nz => write!(f, "nz"),
            CCOperand::c => write!(f, "c"),
            CCOperand::nc => write!(f, "nc"),
            CCOperand::none => Ok(())
        }
    }
}