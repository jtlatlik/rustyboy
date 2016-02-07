use super::instruction::*;
use super::instruction::InstructionType::*;
use super::operands::*;
use super::operands::Reg8Operand::*;
use super::operands::Reg16Operand::*;
use super::operands::Operand::*;
use super::memory::*;

use super::cpu::*;

const OP_LUT : [Operand; 8] = [reg8(b), reg8(c), reg8(d), reg8(e), reg8(h), reg8(l), mem_reg(hl), reg8(a)];
const ALU_LUT : [InstructionType; 8] = [add, adc, sub, sbc, and, xor, or, cp];
const CB_ALU_LUT : [InstructionType; 8] = [rlc, rrc, rl, rr, sla, sra, swap, srl];

macro_rules! insn {
    () => (insn!(invalid));
    ($itype:expr) => (insn!($itype, Operand::none, Operand::none));
    ($itype:expr, $dest:expr) => (insn!($itype, $dest, Operand::none));
    ($itype:expr, $dest:expr, $src:expr) => (insn!($itype, $dest,$src, CCOperand::none));
    ($itype:expr, $dest:expr, $src:expr, $cc:expr) => ({
        
        let length = 1 + match $src {
            imm16(_) | mem_imm(_) => 2,
            imm8(_) | mem_io_imm(_) => 1,
            _ => 0
        } + match $dest {
            imm16(_) | mem_imm(_) => 2,
            imm8(_) |  mem_io_imm(_) => 1,
            _ => 0
        };


        Instruction {
            itype : $itype,
            dest : $dest,
            src : [$src, Operand::none],
            cc : $cc,
            cycles : 4,
            length : length}
    })
}

//impl<M: Memory> CPU<M> {
impl CPU {
    pub fn decode(&self, opcode : [u8; 3]) -> Instruction {
        
        let nn : u16 = ((opcode[2] as u16) << 8) | (opcode[1] as u16); //little endian
        let n : u8 = opcode[1];
        
        match opcode[0] as usize {
            0x00 => insn!(nop),
            0x01 => insn!(ld, reg16(bc), imm16(nn)), 
            0x02 => insn!(ld, mem_reg(bc), reg8(a)),
            0x03 => insn!(inc, reg16(bc)),
            0x04 => insn!(inc, reg8(b)),
            0x05 => insn!(dec, reg8(b)),
            0x06 => insn!(ld, reg8(b), imm8(n)), 
            0x07 => insn!(rlca),
            0x08 => insn!(ld, mem_imm(nn), reg16(sp)),
            0x09 => insn!(add, reg16(hl), reg16(bc)), 
            0x0a => insn!(ld, reg8(a), mem_reg(bc)),
            0x0b => insn!(dec, reg16(bc)),
            0x0c => insn!(inc, reg8(c)),
            0x0d => insn!(dec, reg8(c)),
            0x0e => insn!(ld, reg8(c), imm8(n)),
            0x0f => insn!(rrca),
            0x10 => insn!(stop),
            0x11 => insn!(ld, reg16(de), imm16(nn)), 
            0x12 => insn!(ld, mem_reg(de), reg8(a)),
            0x13 => insn!(inc, reg16(de)),
            0x14 => insn!(inc, reg8(d)),
            0x15 => insn!(dec, reg8(d)),
            0x16 => insn!(ld, reg8(d), imm8(n)), 
            0x17 => insn!(rla),
            0x18 => insn!(jr, none, imm8(n)),
            0x19 => insn!(add, reg16(hl), reg16(de)),
            0x1a => insn!(ld, reg8(a), mem_reg(de)),
            0x1b => insn!(dec, reg16(de)),
            0x1c => insn!(inc, reg8(e)),
            0x1d => insn!(dec, reg8(e)),
            0x1e => insn!(ld, reg8(e), imm8(n)),
            0x1f => insn!(rra),
            0x20 => insn!(jr, none, imm8(n), CCOperand::nz),
            0x21 => insn!(ld, reg16(hl), imm16(nn)), 
            0x22 => insn!(ldi, mem_reg(hl), reg8(a)),
            0x23 => insn!(inc, reg16(hl)),
            0x24 => insn!(inc, reg8(h)),
            0x25 => insn!(dec, reg8(h)),
            0x26 => insn!(ld, reg8(h), imm8(n)), 
            0x27 => insn!(daa),
            0x28 => insn!(jr, none, imm8(n), CCOperand::z),
            0x29 => insn!(add, reg16(hl), reg16(hl)),
            0x2a => insn!(ldi, reg8(a), mem_reg(hl)),
            0x2b => insn!(dec, reg16(hl)),
            0x2c => insn!(inc, reg8(l)),
            0x2d => insn!(dec, reg8(l)),
            0x2e => insn!(ld, reg8(l), imm8(n)),
            0x2f => insn!(cpl),
            0x30 => insn!(jr, none, imm8(n), CCOperand::nc),
            0x31 => insn!(ld, reg16(sp), imm16(nn)), 
            0x32 => insn!(ldd, mem_reg(hl), reg8(a)), 
            0x33 => insn!(inc, reg16(sp)),
            0x34 => insn!(inc, mem_reg(hl)),
            0x35 => insn!(dec, mem_reg(hl)),
            0x36 => insn!(ld, mem_reg(hl), imm8(n)), 
            0x37 => insn!(scf),
            0x38 => insn!(jr, none, imm8(n), CCOperand::c),
            0x39 => insn!(add, reg16(hl), reg16(sp)),
            0x3a => insn!(ldd, reg8(a), mem_reg(hl)),
            0x3b => insn!(dec, reg16(sp)),
            0x3c => insn!(inc, reg8(a)),
            0x3d => insn!(dec, reg8(a)),
            0x3e => insn!(ld, reg8(a), imm8(n)),
            0x3f => insn!(ccf),
            0x76 => insn!(halt),
            op @ 0x40 ... 0x7f => insn!(ld, OP_LUT[(op>>3)&0x7], OP_LUT[op & 0x7]),
            op @ 0x80 ... 0xbf => insn!(ALU_LUT[(op>>3)&0x7], reg8(a), OP_LUT[op & 0x7]),
            0xc0 => insn!(ret, none, none, CCOperand::nz),
            0xc1 => insn!(pop, reg16(bc)),
            0xc2 => insn!(jp, none, imm16(nn), CCOperand::nz),
            0xc3 => insn!(jp, none, imm16(nn)),
            0xc4 => insn!(call, none, imm16(nn), CCOperand::nz),
            0xc5 => insn!(push, none, reg16(bc)),
            0xc6 => insn!(add, reg8(a), imm8(n)),
            0xc7 => insn!(rst, none, imm8(0x00)),
            0xc8 => insn!(ret, none, none, CCOperand::z),
            0xc9 => insn!(ret),
            0xca => insn!(jp, none, imm16(nn), CCOperand::z),
            0xcb => decode_cb_prefix_inst(opcode[1]),
            0xcc => insn!(call, none, imm16(nn), CCOperand::z),
            0xcd => insn!(call, none, imm16(nn)),
            0xce => insn!(adc, reg8(a), imm8(n)),
            0xcf => insn!(rst, none, imm8(0x08)),
            0xd0 => insn!(ret, none, none, CCOperand::nc),
            0xd1 => insn!(pop, reg16(de)),
            0xd2 => insn!(jp, none, imm16(nn), CCOperand::nc),
            0xd3 => insn!(invalid),
            0xd4 => insn!(call, none, imm16(nn), CCOperand::nc),
            0xd5 => insn!(push, none, reg16(de)),
            0xd6 => insn!(sub, reg8(a), imm8(n)),
            0xd7 => insn!(rst, none, imm8(0x10)),
            0xd8 => insn!(ret, none, none, CCOperand::c),
            0xd9 => insn!(reti),
            0xda => insn!(jp, none, imm16(nn), CCOperand::c),
            0xdb => insn!(invalid),
            0xdc => insn!(call, none, imm16(nn), CCOperand::c),
            0xdd => insn!(invalid),
            0xde => insn!(sbc, reg8(a), imm8(n)),
            0xdf => insn!(rst, none, imm8(0x18)),
            0xe0 => insn!(ld, mem_io_imm(n), reg8(a)),
            0xe1 => insn!(pop, reg16(hl)),
            0xe2 => insn!(ld, mem_io_reg(c), reg8(a)),
            0xe3 => insn!(invalid),
            0xe4 => insn!(invalid),
            0xe5 => insn!(push, none, reg16(hl)),
            0xe6 => insn!(and, reg8(a), imm8(n)),
            0xe7 => insn!(rst, none, imm8(0x20)),
            0xe8 => insn!(add, reg16(sp), imm8(n)), //caution: imm8 signed here
            0xe9 => insn!(jp, none, reg16(hl)),
            0xea => insn!(ld, mem_imm(nn), reg8(a)),
            0xeb => insn!(invalid),
            0xec => insn!(invalid),
            0xed => insn!(invalid),
            0xee => insn!(xor, reg8(a), imm8(n)),
            0xef => insn!(rst, none, imm8(0x28)),
            0xf0 => insn!(ld, reg8(a), mem_io_imm(n)),
            0xf1 => insn!(pop, reg16(af)),
            0xf2 => insn!(ld, reg8(a), mem_io_reg(c)),
            0xf3 => insn!(di),
            0xf4 => insn!(invalid),
            0xf5 => insn!(push, none, reg16(af)),
            0xf6 => insn!(or, reg8(a), imm8(n)),
            0xf7 => insn!(rst, none, imm8(0x30)),
            0xf8 => Instruction { src : [reg16(sp), imm8(n)], ..insn!(ld, reg16(hl)) }, //caution: imm8 signed here
            0xf9 => insn!(ld, reg16(sp),reg16(hl)),
            0xfa => insn!(ld, reg8(a), mem_imm(nn)),
            0xfb => insn!(ei),
            0xfc => insn!(invalid),
            0xfd => insn!(invalid),
            0xfe => insn!(cp, reg8(a), imm8(n)),
            0xff => insn!(rst, none, imm8(0x38)),
            _ => unreachable!()
        }
    }
}

fn decode_cb_prefix_inst(opcode : u8) -> Instruction {
    match opcode as usize {
        op @ 0x00 ... 0x3f => insn!(CB_ALU_LUT[(op>>3)&0x7], OP_LUT[op & 0x7]),
        op @ 0x40 ... 0x7f => Instruction { src : [imm8(((op >> 3)&0x7)as u8 ), OP_LUT[op&0x7]] , ..insn!(bit) },
        op @ 0x80 ... 0xbf => insn!(set, OP_LUT[op&0x7], imm8(((op >> 3)&0x7) as u8 )),
        op @ 0xc0 ... 0xff => insn!(res, OP_LUT[op&0x7], imm8(((op >> 3)&0x7) as u8)),
        _ => insn!(invalid)
    }
}