// use std::{thread::sleep, time::Duration};

use crate::processor::architecture;
use super::microarchitecture::*;

pub fn run_meta_program(isa: &mut architecture::ISA, micro_arch: &mut MicroCode) {
    let mut state = 0;

    while isa.BPO {
        use MicroSignal::*;

        match state {
            0 => {
                micro_arch.process_microsignal(&LdMIR);
                state = 1;
            }

            1 => {
                if micro_arch.get_g(isa) == true {
                    micro_arch.process_microsignal(&LdMAR(isa.IR, isa.INTR));
                } else {
                    micro_arch.process_microsignal(&plus1MAR);
                }
                state = 2;
            }

            2 => {
                micro_arch.generate_signals(isa);

                if isa.BE1_CIL == true {
                    isa.BPO = false;
                }

                // isa.print_state(micro_arch);
                // sleep(Duration::from_millis(10));

                state = 0;
            }

            _ => unreachable!(),
        }
    }

    // isa.MEM.print_memory_dump();
}