#![expect(clippy::cast_lossless)]

use crate::processor::{architecture, control_unit::sequencer::Sequencer, signals};  
use num_traits::FromPrimitive;

pub struct MicroCode {
    pub MIR: u64,
    pub MAR: u64,
    pub MPM: Vec<u64>,
}

pub enum MicroSignal {
    LdMAR(u16, bool), // IR, INTR
    plus1MAR,
    LdMIR,
}

pub struct ControlUnit {
    pub micro_code: MicroCode,
    pub sequencer: Sequencer,
}

impl MicroCode {
    pub fn new() -> Self {
        Self {
            MIR: 0,
            MAR: 0,
            MPM: read_microprogram(),
        }
    }

    pub fn get_g(&self, isa: &architecture::ISA) -> bool {
        let succesor_idx = (self.MIR & SUCCESSOR_MASK) >> (SUCCESSOR_MASK.trailing_zeros());
        let t_f_idx = (self.MIR & TRUE_FALSE_MASK) >> (TRUE_FALSE_MASK.trailing_zeros());

        let t_f: bool = t_f_idx != 0;
        let f: bool = match succesor_idx {
            0 => t_f,
            1 => !t_f,
            2 => isa.BE0_ACLOW,
            3 => isa.BE1_CIL,
            4 => isa.get_C(),
            5 => isa.get_Z(),
            6 => isa.get_S(),
            7 => isa.get_V(),
            _ => unreachable!(),
        };

        f ^ t_f
    }

    pub fn get_index(&self, IR: u16, INTR: bool) -> u8 {
        let index_idx = (self.MIR & INDEX_MASK) >> (INDEX_MASK.trailing_zeros());

        match index_idx {
            0 => 0,
            1 => {
                let CL1: bool = get_nth_bit(IR, 15) &  get_nth_bit(IR, 14);
                let CL0: bool = get_nth_bit(IR, 15) & !get_nth_bit(IR, 13);

                (u8::from(CL1)) << 1 | (u8::from(CL1) ^ u8::from(CL0)) // gray to binary
            }
            2 => {
                let IR11 = get_nth_bit(IR, 11);
                let IR10 = get_nth_bit(IR, 10);

                (IR11 as u8) << 1 | (IR10 as u8)
            }
            3 => {
                let IR5 = get_nth_bit(IR, 5);
                let IR4 = get_nth_bit(IR, 4);

                (IR5 as u8) << 1 | (IR4 as u8)
            }
            4 => {
                let IR14 = get_nth_bit(IR, 14);
                let IR13 = get_nth_bit(IR, 13);
                let IR12 = get_nth_bit(IR, 12);

                (IR14 as u8) << 2 | (IR13 as u8) << 1 | (IR12 as u8)
            }
            5 => {
                let IR11 = get_nth_bit(IR, 11);
                let IR10 = get_nth_bit(IR, 10);
                let IR9 = get_nth_bit(IR, 9);
                let IR8 = get_nth_bit(IR, 8);

                (IR11 as u8) << 3 | (IR10 as u8) << 2 | (IR9 as u8) << 1 | (IR8 as u8)
            }
            6 => {
                let IR11 = get_nth_bit(IR, 11);
                let IR10 = get_nth_bit(IR, 10);
                let IR9 = get_nth_bit(IR, 9);
                let IR8 = get_nth_bit(IR, 8);

                (IR11 as u8) << 4 | (IR10 as u8) << 3 | (IR9 as u8) << 2 | (IR8 as u8) << 1
            }
            7 => (INTR as u8) << 2,
            _ => unreachable!(),
        }
    }

    pub fn process_microsignal(&mut self, signal: &MicroSignal) {
        use MicroSignal::*;
        match signal {
            LdMAR(IR, INTR) => self.MAR = (self.MIR & JUMP_ADDR_MASK) + self.get_index(*IR, *INTR) as u64,
            plus1MAR => self.MAR += 1,
            LdMIR => self.MIR = self.MPM[self.MAR as usize],
        }
    }

    pub fn generate_signals(&self, isa: &mut architecture::ISA) {
        let sbus_idx = (self.MIR & SBUS_MASK) >> (SBUS_MASK.trailing_zeros());
        let dbus_idx = (self.MIR & DBUS_MASK) >> (DBUS_MASK.trailing_zeros());
        let alu_idx = (self.MIR & ALU_MASK) >> (ALU_MASK.trailing_zeros());
        let rbus_idx = (self.MIR & RBUS_MASK) >> (RBUS_MASK.trailing_zeros());
        let memory_idx = (self.MIR & MEMORY_MASK) >> (MEMORY_MASK.trailing_zeros());
        let other_idx = (self.MIR & OTHER_MASK) >> (OTHER_MASK.trailing_zeros());

        let sbus_signal: signals::SBUS =
            if let Some(signal) = FromPrimitive::from_u64(sbus_idx) { signal }
            else { isa.BE1_CIL = true; return; };

        let dbus_signal: signals::DBUS =
            if let Some(signal) = FromPrimitive::from_u64(dbus_idx) { signal }
            else { isa.BE1_CIL = true; return; };

        let alu_signal: signals::ALU =
            if let Some(signal) = FromPrimitive::from_u64(alu_idx) { signal }
            else { isa.BE1_CIL = true; return; };

        let rbus_signal: signals::RBUS =
            if let Some(signal) = FromPrimitive::from_u64(rbus_idx) { signal }
            else { isa.BE1_CIL = true; return; };
        
        let memory_signal: signals::Memory =
            if let Some(signal) = FromPrimitive::from_u64(memory_idx) { signal }
            else { isa.BE1_CIL = true; return; };

        let other_signal: signals::Other =
            if let Some(signal) = FromPrimitive::from_u64(other_idx) { signal }
            else { isa.BE1_CIL = true; return; };

        isa.execute(&sbus_signal, &dbus_signal, &rbus_signal, &alu_signal, &memory_signal, &other_signal);
    }
}

impl ControlUnit {
    pub fn new() -> Self {
        Self {
            micro_code: MicroCode::new(),
            sequencer: Sequencer { state: 0 },
        }
    }
}

fn get_nth_bit(value: u16, n: u8) -> bool {
    (value & (1 << n)) != 0
}

const MICROPROGRAM_PATH: &str = "src/processor/control_unit/microprogram";

#[allow(clippy::missing_panics_doc)]
fn read_microprogram() -> Vec<u64> {
    let microprogram = std::fs::read_to_string(MICROPROGRAM_PATH).expect("Failed to read microprogram.txt.");
    microprogram
        .lines()
        .map(|line| line.trim_start_matches("0x").trim())
        .map(|line| u64::from_str_radix(line, 16).expect("Failed to parse microinstruction."))
        .collect()
}

pub const SBUS_MASK:        u64 = 0b1111_0000_0000_0000_0000_0000_0000_0000_0000;
pub const DBUS_MASK:        u64 = 0b0000_1111_0000_0000_0000_0000_0000_0000_0000;
pub const ALU_MASK:         u64 = 0b0000_0000_1111_0000_0000_0000_0000_0000_0000;
pub const RBUS_MASK:        u64 = 0b0000_0000_0000_1111_0000_0000_0000_0000_0000;
pub const MEMORY_MASK:      u64 = 0b0000_0000_0000_0000_1100_0000_0000_0000_0000;
pub const OTHER_MASK:       u64 = 0b0000_0000_0000_0000_0011_1100_0000_0000_0000;
pub const SUCCESSOR_MASK:   u64 = 0b0000_0000_0000_0000_0000_0011_1000_0000_0000;
pub const INDEX_MASK:       u64 = 0b0000_0000_0000_0000_0000_0000_0111_0000_0000;
pub const TRUE_FALSE_MASK:  u64 = 0b0000_0000_0000_0000_0000_0000_0000_1000_0000;
pub const JUMP_ADDR_MASK:   u64 = 0b0000_0000_0000_0000_0000_0000_0000_0111_1111;