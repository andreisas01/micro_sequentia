use crate::cpu::architecture;
use super::microarchitecture::*;

pub fn meta_program(isa: &mut architecture::ISA) {
    let mut state = 0;

    let mut micro_arch = super::microarchitecture::MicroCode::new();

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
                state = 0;
            }

            _ => unreachable!(),
        }

        micro_arch.generate_signals(isa);

        if isa.BE1_CIL == true {
            isa.BPO = false;
        }
    }
}