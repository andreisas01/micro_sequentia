// use std::{thread::sleep, time::Duration};

use crate::processor::architecture;
use super::microarchitecture::*;

pub struct Sequencer {
    pub state: u8,
}

impl Sequencer {
    pub fn tick(&mut self, micro_arch: &mut MicroCode, isa: &mut architecture::ISA) {
        use MicroSignal::*;

        match self.state {
            0 => {
                micro_arch.process_microsignal(&LdMIR);
                self.state = 1;
            }

            1 => {
                if micro_arch.get_g(isa) == true {
                    micro_arch.process_microsignal(&LdMAR(isa.IR, isa.INTR));
                } else {
                    micro_arch.process_microsignal(&plus1MAR);
                }
                self.state = 2;
            }

            2 => {
                micro_arch.generate_signals(isa);

                if isa.BE1_CIL == true {
                    isa.BPO = false;
                }

                // isa.print_state(micro_arch);
                // sleep(Duration::from_millis(10));

                self.state = 0;
            }

            _ => unreachable!(),
        }
    }

    pub fn run_meta_program(&mut self, isa: &mut architecture::ISA, micro_arch: &mut MicroCode) {
        while isa.BPO {
            self.tick(micro_arch, isa);
        }

        // isa.MEM.print_memory_dump();
    }
}

