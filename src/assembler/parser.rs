use crate::{assembler::arhitecture::*, memory};
use std::str::FromStr;

pub struct ParsedCode {
    pub instructions: Vec<AddressableInstruction>,
    pub symbol_table: SymbolTable
}

fn parse_operand(token: &str) -> Result<Operand, String> {
    if let Ok(immediate) = u16::from_str(token) {
        Ok(Operand::Immediate(immediate))
    } else if token.starts_with('(') && token.ends_with(')') {
        let inner = &token[1..token.len() - 1];
        let reg_str = inner
            .strip_prefix('R')
            .ok_or_else(|| format!("Invalid register syntax: {token}"))?;
        let reg_num = u8::from_str(reg_str)
            .map_err(|_| format!("Invalid register: {token}"))?;
        Ok(Operand::Indirect(Register(reg_num)))
    } else if let Some(idx) = token.find('(') {
        if !token.ends_with(')') {
            return Err(format!("Invalid indexed operand: {token}"));
        }

        let reg_part = &token[idx + 1..token.len() - 1];
        let reg_str = reg_part
            .strip_prefix('R')
            .ok_or_else(|| format!("Invalid register syntax: {token}"))?;
        let reg_num = u8::from_str(reg_str)
            .map_err(|_| format!("Invalid register: {token}"))?;
        let offset = u16::from_str(&token[..idx])
            .map_err(|_| format!("Invalid offset: {token}"))?;
        Ok(Operand::Indexed(Register(reg_num), offset))
    } else if let Some(stripped) = token.strip_prefix('R') {
        let reg_num = u8::from_str(stripped)
            .map_err(|_| format!("Invalid register: {token}"))?;
        Ok(Operand::Direct(Register(reg_num)))
    } else {
        Err(format!("Invalid operand: {token}"))
    }
}

fn parse_instruction(line: &str) -> Result<Instruction, String> {
    let tokens: Vec<&str> = line
        .split([' ', ','])
        .filter(|s| !s.is_empty())
        .collect();

    if tokens.is_empty() {
        return Err("Empty instruction line".to_string());
    }

    let opcode = Opcode::from_str(tokens[0])?;

    match opcode {
        Opcode::B1(op) => {
            if tokens.len() < 3 {
                return Err(format!("{} expects 2 operands", tokens[0]));
            }

            let dst = parse_operand(tokens[1])?;
            let src = parse_operand(tokens[2])?;
            if let Operand::Immediate(_) = dst && !matches!(op, OpcodeB1::CMP) {
                return Err(format!("Destination operand cannot be immediate: {}", tokens[1]));
            }
            
            Ok(Instruction::TwoOperands {
                opcode: Opcode::B1(op),
                src,
                dst
            })
        }
        Opcode::B2(op) => {
            if tokens.len() < 2 {
                return Err(format!("{} expects 1 operand", tokens[0]));
            }

            let dst = parse_operand(tokens[1])?;
            if let Operand::Immediate(_) = dst {
                return Err(format!("Destination operand cannot be immediate: {}", tokens[1]));
            }

            Ok(Instruction::OneOperand {
                opcode: Opcode::B2(op),
                dst
            })
        }
        Opcode::B3(op) => {
            if tokens.len() < 2 {
                return Err(format!("{} expects a branch offset", tokens[0]));
            }

            let label = tokens[1].to_string();
            Ok(Instruction::Jump {
                opcode: Opcode::B3(op),
                label
            })
        }
        Opcode::B4(op) => Ok(Instruction::Diverse {
            opcode: Opcode::B4(op)
        }),
    }
}

// comments, multiple-word opcodes and labels
fn normalize_edgecases(input: &str) -> String {
    input.replace("PUSH PC", "PUSH_PC")
         .replace("POP PC", "POP_PC")
         .replace("PUSH FLAG", "PUSH_FLAG")
         .replace("POP FLAG", "POP_FLAG")
         .lines()
         .map(|line| { line.find(['#', ';']).map_or(line, |idx| &line[..idx]) })
         .collect::<Vec<&str>>()
         .join("\n")
}

fn get_instruction_size(instruction: &Instruction) -> Result<InstructionSize, String> {                             
    match instruction {
        Instruction::TwoOperands { src, dst, .. } => {
            let src_extra_size = match src {
                Operand::Immediate(_) | Operand::Indexed(_, _) => true,
                Operand::Direct(_) | Operand::Indirect(_) => false
            };
            let dst_extra_size = match dst {
                Operand::Immediate(_) | Operand::Indexed(_, _) => true,
                Operand::Direct(_) | Operand::Indirect(_) => false
            };
            
            InstructionSize::from_u8(2 + 2 * u8::from(src_extra_size) + 2 * u8::from(dst_extra_size))
        }
        Instruction::OneOperand { dst, .. } => match dst {
            Operand::Immediate(_) | Operand::Indexed(_, _) => Ok(InstructionSize::DoubleWord),
            Operand::Direct(_) | Operand::Indirect(_) => Ok(InstructionSize::Word)
        },
        Instruction::Jump { .. } | Instruction::Diverse { .. } => Ok(InstructionSize::Word)
    }
}

pub fn parse_text(input: &str) -> Result<ParsedCode, String> {
    let mut instructions = Vec::<AddressableInstruction>::new();
    let mut symbol_table = SymbolTable::new();
    let input = normalize_edgecases(input);

    for (line_number, line) in input.lines().enumerate() {
        let mut trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        let address = instructions.last().map_or(memory::START_ADDRESS, |last| last.address + last.size.clone() as u16);

        if let Some((label_part, rest)) = trimmed_line.split_once(':') {
            let label = label_part.trim();
            if !label.is_empty() {
                symbol_table.insert(label.to_string(), address);
            }

            trimmed_line = rest.trim();
            if trimmed_line.is_empty() {
                continue;
            }
        }

        let instruction = parse_instruction(trimmed_line)
            .map_err(|err| format!("Line {}: {err}", line_number + 1))?;
        
        let size = get_instruction_size(&instruction)?;

        // println!("{address:#06X} : {instruction:?}");
        instructions.push(AddressableInstruction {
            instruction,
            address,
            size,
            source_line: line_number + 1,
        });
    }
    Ok(ParsedCode { instructions, symbol_table })
}