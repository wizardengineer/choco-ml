use crate::machine_ir::{MachineBlock, MachineFunc, MachineInstr, VReg};
use ir::{IrFunction, IrInstruction};
use ir::cfg::Literal;
use std::collections::HashMap;

pub fn select_instructions(func: &IrFunction) -> MachineFunc {
    let mut machine_func: MachineFunc = MachineFunc::new(func);

    let mut vreg_mapping: HashMap<String, VReg> = HashMap::new();
    let mut next_vreg = 0;
    let mut allocate_reg = |name: &String| {
        *vreg_mapping.entry(name.clone()).or_insert_with(|| {
            let r = VReg::Virtual(next_vreg);
            next_vreg += 1;
            r
        })
    };

    for block in func.blocks.iter() {
        let mut machine_block: MachineBlock = MachineBlock {
            name: block.label.clone(),
            instrs: Vec::new(),
            succs: block.succs.to_vec(),
        };

        for instr in block.instrs.iter() {
            match instr {
                IrInstruction::Const { dest, value } => {
                    let rd = allocate_reg(dest);
                    let imm = match value {
                        Literal::Int(i) => *i,
                        Literal::Bool(i) => *i as i64,
                    };
                    machine_block.instrs.push(MachineInstr::Li { rd, imm });
                }

                IrInstruction::Assign { lhs, rhs } => {
                    let rd = allocate_reg(lhs);
                    let rs1 = allocate_reg(rhs);
                    machine_block.instrs.push(MachineInstr::Mv { rd, rs1 });
                }

                IrInstruction::Add { dest, lhs, rhs } => {
                    let rd = allocate_reg(dest);
                    let rs1 = allocate_reg(lhs);
                    let rs2 = allocate_reg(rhs);

                    machine_block
                        .instrs
                        .push(MachineInstr::Add { rd, rs1, rs2 });
                }

                IrInstruction::Mul { dest, lhs, rhs } => {
                    let rd = allocate_reg(dest);
                    let rs1 = allocate_reg(lhs);
                    let rs2 = allocate_reg(rhs);

                    machine_block
                        .instrs
                        .push(MachineInstr::Mul { rd, rs1, rs2 });
                }

                IrInstruction::Sub { dest, lhs, rhs } => {
                    let rd = allocate_reg(dest);
                    let rs1 = allocate_reg(lhs);
                    let rs2 = allocate_reg(rhs);

                    machine_block
                        .instrs
                        .push(MachineInstr::Sub { rd, rs1, rs2 });
                }

                IrInstruction::Div { dest, lhs, rhs } => {
                    let rd = allocate_reg(dest);
                    let rs1 = allocate_reg(lhs);
                    let rs2 = allocate_reg(rhs);

                    machine_block
                        .instrs
                        .push(MachineInstr::Div { rd, rs1, rs2 });
                }

                IrInstruction::Call {
                    dest,
                    target_func,
                    args,
                } => {
                    for (i, arg) in args.iter().enumerate() {
                        let src_reg = allocate_reg(arg);
                        if i < 8 {
                            let a_reg = match i {
                                0 => VReg::A0,
                                1 => VReg::A1,
                                2 => VReg::A2,
                                3 => VReg::A3,
                                4 => VReg::A4,
                                5 => VReg::A5,
                                6 => VReg::A6,
                                7 => VReg::A7,
                                _ => unreachable!(),
                            };
                            machine_block.instrs.push(MachineInstr::Mv {
                                rd: a_reg,
                                rs1: src_reg,
                            });
                        } else {
                            let offset = ((i - 8) * 8) as i32;
                            machine_block.instrs.push(MachineInstr::Sw {
                                offset,
                                base: VReg::SP,
                                rs1: src_reg,
                            });
                        }
                    }

                    machine_block.instrs.push(MachineInstr::Jal {
                        rd: VReg::RA,
                        label: target_func.to_string(),
                    });

                    if let Some(d) = dest {
                        let return_value = allocate_reg(d);
                        // A0 is the returh value
                        machine_block.instrs.push(MachineInstr::Mv {
                            rd: return_value,
                            rs1: VReg::A0,
                        });
                    }
                }

                IrInstruction::Br {
                    cond,
                    then_lbl,
                    else_lbl,
                } => {
                    let rs1 = allocate_reg(cond);

                    // if rs1 = 0
                    // goto else_lbl
                    machine_block.instrs.push(MachineInstr::Beqz {
                        rs1,
                        label: else_lbl.to_string(),
                    });

                    // if rs1 = 1, then goto other label (then_lbl)
                    machine_block.instrs.push(MachineInstr::Jmp {
                        label: then_lbl.to_string(),
                    });
                }

                IrInstruction::Jmp { label } => {
                    machine_block.instrs.push(MachineInstr::Jmp {
                        label: label.to_string(),
                    });
                }

                IrInstruction::Ret { args } => {
                    let mut rd = None;

                    if args.is_empty() {
                        rd = Some(allocate_reg(&args[0]));
                    }

                    machine_block.instrs.push(MachineInstr::Ret { rd });
                }

                _ => {}
            }
        }
        machine_func.blocks.push(machine_block.clone());
    }
    machine_func
}
