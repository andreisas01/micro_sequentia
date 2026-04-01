use crate::cpu::architecture;
use super::microarchitecture::*;

pub fn meta_program(isa: &architecture::ISA) {
    let state = 0;

    let mut microcode = super::microarchitecture::MicroCode::new();

    while isa.BPO {
        match state {
            0 => {
                microcode.MIR = microcode.MPM[microcode.MAR as usize];
                state = 1;
            }

            1 => {
                if microcode.get_g(isa) == true {
                    microcode.MAR =(microcode.MIR & JUMP_ADDR_MASK) + isa.get_index();
                    state = 2;
                } else {
                    microcode.MAR += 1;
                }
            }

            2 => {
                state = 3;
            }

            3 => {

            }
        }
    }
}