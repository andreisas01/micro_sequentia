#![expect(non_snake_case, dead_code, clippy::cast_sign_loss, clippy::cast_possible_wrap)]

use super::signals;

pub struct ISA {
    pub MEM: Vec<u16>,
    pub SBUS: u16,
    pub DBUS: u16,
    pub RBUS: u16,
    pub FLAG: u16, // BVI(7)--C(3)--Z(2)--S(1)--V(0)
    pub SP:   u16,
    pub T:    u16,
    pub PC:   u16,
    pub IVR:  u16,
    pub ADR:  u16,
    pub MDR:  u16,
    pub IR:   u16,
    pub RG: [u16; 16],
    pub BPO: bool, // On/Off
    pub BE0_ACLOW: bool, // Exception 0 - Low Power
    pub BE1_CIL: bool, // Exception 1 - Illegal Code
    pub INTR: bool, // Interrupt Request
    pub INTA: bool, // Interrupt Acknowledge
}

impl ISA {
    pub fn new() -> Self {
        Self {
            MEM: vec![0; 65536],
            SBUS: 0,
            DBUS: 0,
            RBUS: 0,
            FLAG: 0,
            SP: 0,
            T: 0,
            PC: 0,
            IVR: 0,
            ADR: 0,
            MDR: 0,
            IR: 0,
            RG: [0; 16],
            BPO: true,
            BE0_ACLOW: false,
            BE1_CIL: false,
            INTR: false,
            INTA: false,
        }
    }

    pub fn execute(&mut self, sbus_signal: &signals::SBUS, dbus_signal: &signals::DBUS, rbus_signal: &signals::RBUS,
                    alu_signal: &signals::ALU, memory_signal: &signals::Memory, other_signal: &signals::Other) {

        let mut Cout: bool = false;
        let mut Zr: bool = false;
        let mut Sr: bool = false;
        let mut DCR: bool = false;

        {   use signals::SBUS::*;
            
            self.SBUS = match sbus_signal {
                None => self.SBUS,
                PdFLAG => self.FLAG,
                PdRG => {
                    let RS = (self.IR & RS_MASK >> RS_MASK.trailing_zeros()) as usize;

                    self.RG[RS]
                },
                PdSP => self.SP,
                PdT => self.T,
                PdTNeg => !self.T,
                PdPC => self.PC,
                PdIVR => self.IVR,
                PdADR => self.ADR,
                PdMDR => self.MDR,
                PdIR_0_7 => self.IR & 0x00FF,
                Pd0 => 0,
                PdNeg1 => u16::MAX,
            };
        }
        {   use signals::DBUS::*;

            self.DBUS = match dbus_signal {
                None => self.DBUS,
                PdFLAG => self.FLAG,
                PdRG => {
                    let RD = (self.IR & RD_MASK >> RD_MASK.trailing_zeros()) as usize;

                    self.RG[RD]
                },
                PdSP => self.SP,
                PdT => self.T,
                PdPC => self.PC,
                PdIVR => self.IVR,
                PdADR => self.ADR,
                PdMDR => self.MDR,
                PdMDRNeg => !self.MDR,
                PdIR_0_7 => self.IR & 0x00FF,
                Pd0 => 0,
                PdNeg1 => u16::MAX,
            };
        }
        {   use signals::ALU::*;

            self.RBUS = match alu_signal {
                None => self.RBUS,
                SBUS => self.SBUS,
                DBUS => self.DBUS,
                SUM => {
                    Cout = (self.SBUS as u32 + self.DBUS as u32) > u16::MAX as u32;
                    DCR = !((self.SBUS & 0x8000 != 0) ^ (self.DBUS & 0x8000 != 0)) & // signS !^ signD   &
                        (((self.SBUS + self.DBUS) & 0x8000 != 0) ^ Cout);           // signR ^ carryOut

                    self.SBUS + self.DBUS
                },
                SUB => {
                    Cout = (self.SBUS as u32 - self.DBUS as u32) > u16::MAX as u32;
                    DCR = !((self.SBUS & 0x8000 != 0) ^ (self.DBUS & 0x8000 != 0)) & // signS !^ signD   &
                        (((self.SBUS - self.DBUS) & 0x8000 != 0) ^ Cout);           // signR ^ carryOut
                    
                    self.SBUS - self.DBUS
                },
                AND => self.SBUS & self.DBUS,
                OR => self.SBUS | self.DBUS,
                XOR => self.SBUS ^ self.DBUS,
                ASL => {
                    let bit15: bool = ((self.DBUS >> 15) & 1) != 0;
                    self.set_C(bit15);

                    self.DBUS << 1
                },
                ASR => {
                    let bit0: bool = (self.DBUS & 1) != 0;
                    self.set_C(bit0);

                    ((self.DBUS as i16) >> 1) as u16
                },
                LSR => {
                    let bit0: bool = (self.DBUS & 1) != 0;
                    self.set_C(bit0);

                    self.DBUS >> 1
                },
                ROL => {
                    let bit15: bool = ((self.DBUS >> 15) & 1) != 0;
                    self.set_C(bit15);

                    self.DBUS.rotate_left(1)
                },
                ROR => {
                    let bit0: bool = (self.DBUS & 1) != 0;
                    self.set_C(bit0);

                    self.DBUS.rotate_right(1)
                },
                RLC => {
                    let bit15: bool = ((self.DBUS >> 15) & 1) != 0;
                    let carry = self.get_C();
                    self.set_C(bit15);

                    (self.DBUS << 1) | (carry as u16)
                },
                RRC => {
                    let bit0: bool = (self.DBUS & 1) != 0;
                    let carry = self.get_C();
                    self.set_C(bit0);

                    (self.DBUS >> 1) | ((carry as u16) << 15)
                },
            };

            if self.RBUS == 0           { Zr = true; }
            if self.RBUS & 0x8000 != 0  { Sr = true; }
        }
        {   use signals::Other::*;

            match other_signal {
                None => {},
                plus2SP => self.SP += 2,
                minus2SP => self.SP -= 2,
                plus2PC => self.PC += 2,
                A1BE0 => self.BE0_ACLOW = true,
                A1BE1 => self.BE1_CIL = true,
                PdCONDaritm => {
                    self.set_C(Cout);
                    self.set_Z(Zr);
                    self.set_S(Sr);
                    self.set_V(DCR);
                },
                Cin_PdCONDaritm => {
                    self.RBUS += 1;
                    self.set_C(Cout);
                    self.set_Z(Zr);
                    self.set_S(Sr);
                    self.set_V(DCR);
                },
                PdCONDlog => {
                    self.set_Z(Zr);
                    self.set_S(Sr);
                },
                A1BVI => self.set_BVI(true),
                A0BVI => self.set_BVI(false),
                A0BPO => self.BPO = false,
                INTA_minus2SP => {
                    self.INTA = true;
                    self.SP -= 2;
                },
                A0BE_A0BI => {
                    self.BE0_ACLOW = false;
                    self.BE1_CIL = false;
                    self.set_BVI(false);
                },
            }
        }
        {   use signals::Memory::*;

            match memory_signal {
                None => {},
                IFCH => self.IR = self.MEM[self.ADR as usize],
                READ => self.MDR = self.MEM[self.ADR as usize],
                WRITE => self.MEM[self.ADR as usize] = self.RBUS,
            }

        }
        {   use signals::RBUS::*;

            match rbus_signal {
                None => {},
                PmFLAG => self.FLAG = self.RBUS,
                PmFLAG_3_0 => self.FLAG = (self.FLAG & 0b1111_0000) | (self.RBUS & 0b0000_1111),
                PmRG => {
                    let RD = (self.IR & RD_MASK >> RD_MASK.trailing_zeros()) as usize;
                    self.RG[RD] = self.RBUS;
                },
                PmSP => self.SP = self.RBUS,
                PmT => self.T = self.RBUS,
                PmPC => self.PC = self.RBUS,
                PmIVR => self.IVR = self.RBUS,
                PmADR => self.ADR = self.RBUS,
                PmMDR => self.MDR = self.RBUS,
            }
        }
    }

    // -- getters and setters for FLAG bits --

    fn get_BVI(&self) -> bool {
        (self.FLAG & 0b1000_0000) != 0
    }

    fn set_BVI(&mut self, value: bool) {
        if value {
            self.FLAG |= 0b1000_0000;
        } else {
            self.FLAG &= !0b1000_0000;
        }
    }

    pub fn get_C(&self) -> bool {
        (self.FLAG & 0b0000_1000) != 0
    }

    fn set_C(&mut self, value: bool) {
        if value {
            self.FLAG |= 0b0000_1000;
        } else {
            self.FLAG &= !0b0000_1000;
        }
    }

    pub fn get_Z(&self) -> bool {
        (self.FLAG & 0b0000_0100) != 0
    }

    fn set_Z(&mut self, value: bool) {
        if value {
            self.FLAG |= 0b0000_0100;
        } else {
            self.FLAG &= !0b0000_0100;
        }
    }

    pub fn get_S(&self) -> bool {
        (self.FLAG & 0b0000_0010) != 0
    }

    fn set_S(&mut self, value: bool) {
        if value {
            self.FLAG |= 0b0000_0010;
        } else {
            self.FLAG &= !0b0000_0010;
        }
    }

    pub fn get_V(&self) -> bool {
        (self.FLAG & 0b0000_0001) != 0
    }

    fn set_V(&mut self, value: bool) {
        if value {
            self.FLAG |= 0b0000_0001;
        } else {
            self.FLAG &= !0b0000_0001;
        }
    }

    // -- end of region for FLAG bits --
}

pub const RS_MASK: u16 = 0b0000_0011_1100_0000;
pub const RD_MASK: u16 = 0b0000_0000_0000_1111;