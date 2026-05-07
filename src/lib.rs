// micro_sequentia library crate

#![expect(missing_debug_implementations, missing_copy_implementations, clippy::new_without_default,
    clippy::cast_possible_truncation, clippy::bool_comparison, clippy::cast_lossless, non_snake_case,
    clippy::uninlined_format_args, unused_crate_dependencies)]
#![allow(clippy::enum_glob_use)]

pub mod assembler {
    #![expect(missing_debug_implementations, clippy::cast_possible_truncation, clippy::needless_range_loop,
            clippy::cast_sign_loss, clippy::cast_possible_wrap)]

    pub(super) mod arhitecture; // internal module
    pub mod parser;
    pub mod encoder;
}

pub mod processor {
    #![expect(non_camel_case_types)]

    use crate::memory;
    use num_traits::FromPrimitive;

    pub mod architecture;
    pub mod signals;

    pub mod control_unit {
        pub mod sequencer;
        pub(super) mod microarchitecture;
    }

    #[derive(Debug, Clone)]
    pub struct CpuSnapshot {
        pub mir: u64,
        pub mar: u64,
        pub sequencer_state: u8,
        pub sbus: u16,
        pub dbus: u16,
        pub rbus: u16,
        pub flag: u16,
        pub sp: u16,
        pub t: u16,
        pub pc: u16,
        pub ivr: u16,
        pub adr: u16,
        pub mdr: u16,
        pub ir: u16,
        pub registers: [u16; 16],
        pub bpo: bool,
        pub be0_aclow: bool,
        pub be1_cil: bool,
        pub intr: bool,
        pub inta: bool,
    }

    #[derive(Debug, Clone)]
    pub struct MirSignalsSnapshot {
        pub sbus: String,
        pub dbus: String,
        pub alu: String,
        pub rbus: String,
        pub memory: String,
        pub other: String,
    }

    pub struct CPU {
        pub control_unit: control_unit::microarchitecture::ControlUnit,
        pub isa: architecture::ISA,
    }

    impl CPU {
        pub fn new(memory: memory::MemoryInterface) -> Self {
            Self {
                control_unit: control_unit::microarchitecture::ControlUnit::new(),
                isa: architecture::ISA::new(memory),
            }
        }

        // only used for the CLI
        pub fn run(&mut self, frequency_hz: u32) {
            self.control_unit.sequencer.run_meta_program(&mut self.isa, &mut self.control_unit.micro_code, frequency_hz);
        }

        // only used for the GUI
        pub fn tick(&mut self) {
            self.control_unit.sequencer.tick(&mut self.control_unit.micro_code, &mut self.isa, false);
        }

        pub fn is_halted(&self) -> bool {
            !self.isa.BPO
        }

        pub fn microprogram(&self) -> &[u64] {
            &self.control_unit.micro_code.MPM
        }

        pub fn snapshot(&self) -> CpuSnapshot {
            CpuSnapshot {
                mir: self.control_unit.micro_code.MIR,
                mar: self.control_unit.micro_code.MAR,
                sequencer_state: self.control_unit.sequencer.state,
                sbus: self.isa.SBUS,
                dbus: self.isa.DBUS,
                rbus: self.isa.RBUS,
                flag: self.isa.FLAG,
                sp: self.isa.SP,
                t: self.isa.T,
                pc: self.isa.PC,
                ivr: self.isa.IVR,
                adr: self.isa.ADR,
                mdr: self.isa.MDR,
                ir: self.isa.IR,
                registers: self.isa.RG,
                bpo: self.isa.BPO,
                be0_aclow: self.isa.BE0_ACLOW,
                be1_cil: self.isa.BE1_CIL,
                intr: self.isa.INTR,
                inta: self.isa.INTA,
            }
        }

        pub fn mir_signals_snapshot(&self) -> MirSignalsSnapshot {
            use control_unit::microarchitecture::{
                ALU_MASK, DBUS_MASK, MEMORY_MASK, OTHER_MASK, RBUS_MASK, SBUS_MASK,
            };

            let mir = self.control_unit.micro_code.MIR;

            let sbus_idx = (mir & SBUS_MASK) >> SBUS_MASK.trailing_zeros();
            let dbus_idx = (mir & DBUS_MASK) >> DBUS_MASK.trailing_zeros();
            let alu_idx = (mir & ALU_MASK) >> ALU_MASK.trailing_zeros();
            let rbus_idx = (mir & RBUS_MASK) >> RBUS_MASK.trailing_zeros();
            let memory_idx = (mir & MEMORY_MASK) >> MEMORY_MASK.trailing_zeros();
            let other_idx = (mir & OTHER_MASK) >> OTHER_MASK.trailing_zeros();

            let sbus = FromPrimitive::from_u64(sbus_idx)
                .map_or_else(|| format!("INVALID({sbus_idx})"), |signal: signals::SBUS| format!("{signal:?}"));
            let dbus = FromPrimitive::from_u64(dbus_idx)
                .map_or_else(|| format!("INVALID({dbus_idx})"), |signal: signals::DBUS| format!("{signal:?}"));
            let alu = FromPrimitive::from_u64(alu_idx)
                .map_or_else(|| format!("INVALID({alu_idx})"), |signal: signals::ALU| format!("{signal:?}"));
            let rbus = FromPrimitive::from_u64(rbus_idx)
                .map_or_else(|| format!("INVALID({rbus_idx})"), |signal: signals::RBUS| format!("{signal:?}"));
            let memory = FromPrimitive::from_u64(memory_idx)
                .map_or_else(|| format!("INVALID({memory_idx})"), |signal: signals::Memory| format!("{signal:?}"));
            let other = FromPrimitive::from_u64(other_idx)
                .map_or_else(|| format!("INVALID({other_idx})"), |signal: signals::Other| format!("{signal:?}"));

            MirSignalsSnapshot {
                sbus,
                dbus,
                alu,
                rbus,
                memory,
                other,
            }
        }
    }
}

pub mod memory;