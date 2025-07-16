use ir::{BlockID, IrFunction};
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct MachineFunc {
    pub name: String,
    pub args: Vec<VReg>,
    pub blocks: Vec<MachineBlock>,
    pub label_to_idx: HashMap<String, usize>,
}

impl MachineFunc {
    pub fn new(func: &IrFunction) -> Self {
        Self {
            name: func.name.to_string(),
            args: Vec::new(),
            blocks: Vec::new(),
            label_to_idx: func.label_to_idx.clone(),
        }
    }

    pub fn block_index(&self, label: &String) -> Option<usize> {
        self.label_to_idx.get(label).copied()
    }
}

#[derive(Debug, Clone)]
pub struct MachineBlock {
    pub name: String,
    pub instrs: Vec<MachineInstr>,
    pub succs: Vec<BlockID>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum VReg {
    Virtual(i32),
    // Temp registers
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,

    // Function arguments
    A0, // function argument 0 / return value 0
    A1, // function argument 1 / return value 1
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,

    // Saved registers
    S0, // frame pointer
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,

    // Return address value
    RA,

    // Stack pointer & Frame pointer
    SP,
    FP,

    // Global Register
    GP,
}

impl VReg {
    pub fn name(&self) -> String {
        match self {
            VReg::T0 => "t0".to_string(),
            VReg::T1 => "t1".to_string(),
            VReg::T2 => "t2".to_string(),
            VReg::T3 => "t3".to_string(),
            VReg::T4 => "t4".to_string(),
            VReg::T5 => "t5".to_string(),
            VReg::T6 => "t6".to_string(),

            VReg::A0 => "a0".to_string(),
            VReg::A1 => "a1".to_string(),
            VReg::A2 => "a2".to_string(),
            VReg::A3 => "a3".to_string(),
            VReg::A4 => "a4".to_string(),
            VReg::A5 => "a5".to_string(),
            VReg::A6 => "a6".to_string(),
            VReg::A7 => "a7".to_string(),

            VReg::S0 => "s0".to_string(),

            VReg::S1 => "s1".to_string(),
            VReg::S2 => "s2".to_string(),
            VReg::S3 => "s3".to_string(),
            VReg::S4 => "s4".to_string(),
            VReg::S5 => "s5".to_string(),
            VReg::S6 => "s6".to_string(),
            VReg::S7 => "s7".to_string(),
            VReg::S8 => "s8".to_string(),
            VReg::S9 => "s9".to_string(),
            VReg::S10 => "s10".to_string(),
            VReg::S11 => "s11".to_string(),

            VReg::SP => "sp".to_string(),
            VReg::RA => "ra".to_string(),
            VReg::GP => "gp".to_string(),
            VReg::FP => "fp".to_string(),
            _ => "rt".to_string(),
        }
    }
}

/// Machine Instructions, 1:1 to RiscV
#[derive(Debug, Clone)]
pub enum MachineInstr {
    // R1 = R2 + Imm
    Addi { rd: VReg, rs1: VReg, imm: i64 },

    Add { rd: VReg, rs1: VReg, rs2: VReg },

    Mul { rd: VReg, rs1: VReg, rs2: VReg },

    Sub { rd: VReg, rs1: VReg, rs2: VReg },

    Div { rd: VReg, rs1: VReg, rs2: VReg },

    // Load & Store
    Li { rd: VReg, imm: i64 },

    Mv { rd: VReg, rs1: VReg },

    Sw { rs1: VReg, offset: i32, base: VReg },

    // Control flow Instructions
    // May not be needed? Seems we can use
    // Pseudoinstructions like Call or Ret
    Jal { rd: VReg, label: String },

    // Unconditional jump
    Jmp { label: String },

    Beqz { rs1: VReg, label: String },

    Beq { rs1: VReg, rs2: VReg, label: String },

    Ret { rd: Option<VReg> },

    Call { func: String },

    Print { args: Vec<VReg> },
    // TODO: Add more instructions
}

impl MachineInstr {
    pub fn defs(&self) -> Vec<VReg> {
        match self {
            MachineInstr::Add { rd, .. }
            | MachineInstr::Addi { rd, .. }
            | MachineInstr::Mul { rd, .. }
            | MachineInstr::Sub { rd, .. }
            | MachineInstr::Div { rd, .. }
            | MachineInstr::Mv { rd, .. }
            | MachineInstr::Li { rd, .. } => {
                vec![*rd]
            }
            _ => Vec::new(),
        }
    }

    pub fn uses(&self) -> Vec<VReg> {
        match self {
            MachineInstr::Add { rs1, rs2, .. }
            | MachineInstr::Mul { rs1, rs2, .. }
            | MachineInstr::Sub { rs1, rs2, .. }
            | MachineInstr::Beq { rs1, rs2, .. }
            | MachineInstr::Div { rs1, rs2, .. } => {
                vec![*rs1, *rs2]
            }

            MachineInstr::Addi { rs1, .. }
            | MachineInstr::Sw { rs1, .. }
            | MachineInstr::Beqz { rs1, .. }
            | MachineInstr::Mv { rs1, .. } => {
                vec![*rs1]
            }

            _ => Vec::new(),
        }
    }
}
