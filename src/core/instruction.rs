use std::fmt;

use super::operands::*;
use self::InstructionType::*;

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum InstructionType {
    //Loadcommands
    ld,
    ldi,
    ldd,
    push,
    pop,
    //Arithmetic/logical Commands
    add,
    adc,
    sub,
    sbc,
    and,
    xor,
    or,
    cp,
    inc,
    dec,
    daa,
    cpl,
    //Rotate and Shift Commands
    rlca,
    rla,
    rrca,
    rra,
    rl,
    rlc,
    rr,
    rrc,
    sla,
    sra,
    srl,
    swap,
    //Singlebit Operation Commands
    bit,
    set,
    res,
    //CPU-Controlcommands
    ccf,
    scf,
    nop,
    halt,
    stop,
    di,
    ei,
    //branches
    jp,
    jr,
    call,
    ret,
    reti,
    rst,
    //
    invalid
}

#[derive(Copy,Clone)]
pub struct Instruction {
    pub itype : InstructionType,
    pub dest : Operand,
    pub src : [Operand; 2],
    pub cc : CCOperand,
    pub length : u16
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{}\t", self.itype, self.cc);

        match self.itype {
            invalid | nop | ret | reti | di | ei | halt| stop => Ok(()),
            jp | jr | call | push | rst => write!(f, "{}", self.src[0]),
            inc | dec | pop => write!(f, "{}", self.dest),
            bit | set | res => write!(f, "{}, {}", self.src[1], self.src[0]),
            _ => write!(f, "{}, {}", self.dest, self.src[0])
        }
    }
}