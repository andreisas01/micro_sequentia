#![expect(non_snake_case, dead_code)]

pub struct ISA {
    SBUS: u16,
    DBUS: u16,
    RBUS: u16,
    FLAG: u16, // BVI(7)--C(3)--Z(2)--S(1)--V(0)
    SP:   u16,
    T:    u16,
    PC:   u16,
    IVR:  u16,
    ADR:  u16,
    MDR:  u16,
    IR:   u16,
    RG: [u16; 16],
    pub BPO: bool,
    pub ACLOW: bool,
    pub CIL: bool
}

impl ISA {
    pub fn new() -> Self {
        Self {
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
            ACLOW: false,
            CIL: false
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