// micro_sequentia library crate

#![expect(missing_debug_implementations, missing_copy_implementations, clippy::new_without_default,
    clippy::cast_possible_truncation, clippy::bool_comparison, clippy::cast_lossless, non_snake_case,
    clippy::uninlined_format_args)]
#![allow(clippy::enum_glob_use)]

pub mod assembler {
    #![expect(missing_debug_implementations, clippy::cast_possible_truncation)]

    pub(super) mod arhitecture; // internal module
    pub mod parser;
    pub mod encoder;
}

pub mod processor {
    #![expect(non_camel_case_types)]

    use crate::memory;

    pub mod architecture;
    pub mod signals;

    pub mod control_unit {
        pub mod sequencer;
        pub(super) mod microarchitecture;
    }

    pub struct CPU {
        control_unit: control_unit::microarchitecture::MicroCode,
        isa: architecture::ISA,
    }

    impl CPU {
        pub fn new(memory: memory::MemoryInterface) -> Self {
            Self {
                control_unit: control_unit::microarchitecture::MicroCode::new(),
                isa: architecture::ISA::new(memory),
            }
        }

        pub fn run(&mut self) {
            control_unit::sequencer::run_meta_program(&mut self.isa, &mut self.control_unit);
        }
    }
}

pub mod memory;