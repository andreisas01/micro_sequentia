#![expect(missing_copy_implementations)]

use std::str::FromStr;
use strum::EnumCount;
use strum_macros::{EnumCount, EnumString};

#[derive(Debug)]
pub enum Instruction {
    // B1
    TwoOperands {
        opcode: Opcode,
        src: Operand,
        dst: Operand
    },

    // B2
    OneOperand {
        opcode: Opcode,
        dst: Operand
    },

    // B3
    Jump {
        opcode: Opcode,
        label: String
    },

    // B4
    Diverse {
        opcode: Opcode
    }
}

#[derive(Debug)]
pub enum Operand {
    Immediate(u16), // AM
    Direct(Register), // AD
    Indirect(Register), // AI
    Indexed(Register, u16) // AX
}

#[derive(Debug)]
pub struct Register(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    B1(OpcodeB1),
    B2(OpcodeB2),
    B3(OpcodeB3),
    B4(OpcodeB4),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumCount)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
pub enum OpcodeB1 {
    MOV,
    ADD,
    SUB,
    CMP,
    AND,
    OR,
    XOR
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumCount)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
pub enum OpcodeB2 {
    CLR,
    NEG,
    INC,
    DEC,
    ASL,
    ASR,
    LSR,
    ROL,
    ROR,
    RLC,
    RRC,
    JMP,
    CALL,
    PUSH,
    POP
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumCount)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
pub enum OpcodeB3 {
    BR,
    BNE,
    BEQ,
    BPL,
    BMI,
    BCS,
    BCC,
    BVS,
    BVC
}

#[expect(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumCount)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
pub enum OpcodeB4 {
    CLC,
    CLV,
    CLZ,
    CLS,
    CCC,
    SEC,
    SEV,
    SEZ,
    SES,
    SCC,
    NOP,
    RET,
    RETI,
    HALT,
    WAIT,
    PUSH_PC,
    POP_PC,
    PUSH_FLAG,
    POP_FLAG
}

pub const START_ADDRESS: u16 = 0x0000;

pub struct AddressableInstruction {
    pub instruction: Instruction,
    pub address: u16,
    pub size: InstructionSize
}

pub type SymbolTable = std::collections::HashMap<String, u16>;

#[derive(Clone, Debug)]
pub enum InstructionSize { Word = 1, DoubleWord = 2, TripleWord = 3 }

impl InstructionSize {
    pub fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(InstructionSize::Word),
            2 => Ok(InstructionSize::DoubleWord),
            3 => Ok(InstructionSize::TripleWord),
            _ => Err(format!("Invalid instruction size: {value}")),
        }
    }
}

impl Opcode {
    pub fn from_str(opcode_str: &str) -> Result<Self, String> {
        <Self as FromStr>::from_str(opcode_str)
    }

    pub fn to_u16(self) -> u16 {
        match self {
            Opcode::B1(op) => op as u16,
            Opcode::B2(op) => (op as u16) + OpcodeB1::COUNT as u16,
            Opcode::B3(op) => (op as u16) + OpcodeB1::COUNT as u16 + OpcodeB2::COUNT as u16,
            Opcode::B4(op) => op as u16 + OpcodeB1::COUNT as u16 + OpcodeB2::COUNT as u16 + OpcodeB3::COUNT as u16,
        }
    }
}

impl FromStr for Opcode {
    type Err = String;

    fn from_str(opcode_str: &str) -> Result<Self, Self::Err> {
        if let Ok(op) = opcode_str.parse::<OpcodeB1>() {
            return Ok(Self::B1(op));
        }
        if let Ok(op) = opcode_str.parse::<OpcodeB2>() {
            return Ok(Self::B2(op));
        }
        if let Ok(op) = opcode_str.parse::<OpcodeB3>() {
            return Ok(Self::B3(op));
        }
        if let Ok(op) = opcode_str.parse::<OpcodeB4>() {
            return Ok(Self::B4(op));
        }

        Err(format!("Unknown opcode: {opcode_str}"))
    }
}