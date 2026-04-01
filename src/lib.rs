// micro_sequentia library crate

pub mod assembler {
    #![expect(missing_debug_implementations, clippy::cast_possible_truncation)]

    pub(super) mod arhitecture; // internal module
    pub mod parser;
    pub mod encoder;
}

pub mod cpu {
    pub mod architecture;

    pub mod control_unit {
        #![expect(non_snake_case)]

        pub mod sequencer;
        pub(super) mod microarchitecture;
    }

    pub struct CPU {
        control_unit: control_unit::microarchitecture::MicroCode,
        isa: architecture::ISA,
    }
}