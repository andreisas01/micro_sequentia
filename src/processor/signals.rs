use num_derive::FromPrimitive;    

#[derive(Debug, FromPrimitive)]
pub enum SBUS {
    None,
    PdFLAG,
    PdRG,
    PdSP,
    PdT,
    PdTNeg,
    PdPC,
    PdIVR,
    PdADR,
    PdMDR,
    PdIR_0_7,
    Pd0,
    PdNeg1,
}

#[derive(Debug, FromPrimitive)]
pub enum DBUS {
    None,
    PdFLAG,
    PdRG,
    PdSP,
    PdT,
    PdPC,
    PdIVR,
    PdADR,
    PdMDR,
    PdMDRNeg,
    PdIR_0_7,
    Pd0,
    PdNeg1,
}

#[derive(Debug, FromPrimitive)]
pub enum ALU {
    None,
    SBUS,
    DBUS,
    SUM,
    SUB,
    AND,
    OR,
    XOR,
    ASL,
    ASR,
    LSR,
    ROL,
    ROR,
    RLC,
    RRC,
}

#[derive(Debug, FromPrimitive)]
pub enum RBUS {
    None,
    PmFLAG,
    PmFLAG_3_0,
    PmRG,
    PmSP,
    PmT,
    PmPC,
    PmIVR,
    PmADR,
    PmMDR,
}

#[derive(Debug, FromPrimitive)]
pub enum Memory {
    None,
    IFCH,
    READ,
    WRITE,
}

#[derive(Debug, FromPrimitive, PartialEq, Eq)]
pub enum Other {
    None,
    plus2SP,
    minus2SP,
    plus2PC,
    A1BE0,
    A1BE1,
    PdCONDaritm,
    Cin_PdCONDaritm,
    PdCONDlog,
    A1BVI,
    A0BVI,
    A0BPO,
    INTA_minus2SP,
    A0BE_A0BI,
}