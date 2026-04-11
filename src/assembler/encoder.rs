#![expect(clippy::similar_names, clippy::cast_lossless)]

use crate::assembler::{arhitecture::*, parser::ParsedCode};
use strum::EnumCount;

pub struct OpcodeTable(Vec<u16>);

// returns little-endian byte representation of the instruction
fn encode_instruction(instr: AddressableInstruction,
    symbol_table: &SymbolTable, bit_table: &OpcodeTable) -> Vec<u8> {

    let mut words = Vec::<u16>::new();
    match instr.instruction {
        Instruction::TwoOperands { opcode, src, dst } => {
            let mut word = bit_table.0[opcode.to_u16() as usize];

            let (mas, rs) = match src {
                Operand::Immediate(_) => (0b00, 0b0000),
                Operand::Direct(Register(reg)) => (0b01, reg as u16),
                Operand::Indirect(Register(reg)) => (0b10, reg as u16),
                Operand::Indexed(Register(reg), _) => (0b11, reg as u16)
            };
            let (mad, rd) = match dst {
                Operand::Immediate(_) => (0b00, 0b0000),
                Operand::Direct(Register(reg)) => (0b01, reg as u16),
                Operand::Indirect(Register(reg)) => (0b10, reg as u16),
                Operand::Indexed(Register(reg), _) => (0b11, reg as u16)
            };

            word |= mas << 10 | rs << 6 | mad << 4 | rd;
            words.push(word);

            // handle extra words for AM and AX addressing modes
            if let Operand::Immediate(imm) = src {
                words.push(imm);
            }
            else if let Operand::Indexed(_, offset) = src {
                words.push(offset);
            }

            if let Operand::Immediate(imm) = dst {
                words.push(imm);
            }
            else if let Operand::Indexed(_, offset) = dst {
                words.push(offset);
            }
        }
        Instruction::OneOperand { opcode, dst } => {
            let mut word = bit_table.0[opcode.to_u16() as usize];

            let (mad, rd) = match dst {
                Operand::Immediate(_) => (0b00, 0b0000),
                Operand::Direct(Register(reg)) => (0b01, reg as u16),
                Operand::Indirect(Register(reg)) => (0b10, reg as u16),
                Operand::Indexed(Register(reg), _) => (0b11, reg as u16)
            };

            word |=  mad << 4 | rd;
            words.push(word);

            if let Operand::Immediate(imm) = dst {
                words.push(imm);
            }
            else if let Operand::Indexed(_, offset) = dst {
                words.push(offset);
            }
        }
        Instruction::Jump { opcode, label } => {
            let mut word = bit_table.0[opcode.to_u16() as usize];

            let target_address = symbol_table.get(&label).unwrap();
            // relative to PC
            let offset = (*target_address as i16 - (instr.address + instr.size as u16) as i16) as u16 & 0x00FF;

            word |= offset;
            words.push(word);
        }
        Instruction::Diverse { opcode } => {
            words.push(bit_table.0[opcode.to_u16() as usize]);
        }
    }

    let mut bytes = Vec::<u8>::new();
    for word in words {
        bytes.push((word & 0xFF) as u8);
        bytes.push((word >> 8) as u8);
    }
    bytes
}

pub fn encode_parsed_code(parsed_code: ParsedCode, bit_table: &OpcodeTable) -> Vec<u8> {
    let mut encoded = Vec::new();

    for instr in parsed_code.instructions {
        encoded.extend_from_slice(
            &encode_instruction(instr, &parsed_code.symbol_table, bit_table));
    }

    encoded
}

impl OpcodeTable {
    pub fn generate() -> Self {
        let mut bit_table = Vec::new();
        let B4_opcodes = Opcode::get_B4_opcodes();

        for i in 0..OpcodeB1::COUNT {
            bit_table.push((i << 12) as u16);
        }
        for i in 0..OpcodeB2::COUNT {
            bit_table.push((i << 8 | 0x8000) as u16);
        }
        for i in 0..OpcodeB3::COUNT {
            bit_table.push((i << 8 | 0xC000) as u16);
        }
        for i in 0..OpcodeB4::COUNT {
            bit_table.push(B4_opcodes[i]);
        }

        OpcodeTable(bit_table)
    }
}