use crate::cpu::architecture;

pub struct MicroCode {
    pub MIR: u64,
    pub MAR: u64,
    pub MPM: Vec<u64>,
}

impl MicroCode {
    pub fn new() -> Self {
        Self {
            MIR: 0,
            MAR: 0,
            MPM: read_microprogram(),
        }
    }

    pub fn get_g(self, isa: &architecture::ISA) -> bool {
        let succesor = (self.MIR & SUCCESSOR_MASK) >> (SUCCESSOR_MASK.trailing_zeros());
        let t_f: bool = (self.MIR & TRUE_FALSE_MASK) >> (TRUE_FALSE_MASK.trailing_zeros()) != 0;

        let f: bool = match succesor {
            0 => t_f,
            1 => !t_f,
            2 => isa.ACLOW,
            3 => isa.CIL,
            4 => isa.get_C(),
            5 => isa.get_Z(),
            6 => isa.get_S(),
            7 => isa.get_V(),
            _ => unreachable!(),
        };

        f ^ t_f
    }
}



#[allow(clippy::missing_panics_doc)]
fn read_microprogram() -> Vec<u64> {
    let microprogram = std::fs::read_to_string("microprogram").expect("Failed to read microprogram.txt.");
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