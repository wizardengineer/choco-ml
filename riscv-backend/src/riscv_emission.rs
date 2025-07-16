use crate::machine_ir::*;
use crate::register_alloc::{LinearScan, LiveIntervals};
use crate::VReg;
use std::collections::HashMap;

// In case we manual added a register into our system, that hasn't been
// processed through our live intervals, then we'll nudge the compiler to know
// it's okay to use the Register.
//
// For things like calling conventions, this will be really useful
fn to_phys(v: VReg, map: &HashMap<VReg, LiveIntervals>) -> VReg {
    match v {
        VReg::A0
        | VReg::A1
        | VReg::A2
        | VReg::A3
        | VReg::A4
        | VReg::A5
        | VReg::A6
        | VReg::A7
        | VReg::RA
        | VReg::SP
        | VReg::FP
        | VReg::GP => v, // hardware reg → remain itself

        // otherwise, look up your real virtual regs:
        _ => map.get(&v).and_then(|iv| iv.phy_reg).unwrap_or(v),
    }
}

pub fn emit_riscv(module: &[MachineFunc]) {
    let mut allocator = LinearScan::new();
    let func_by_intervals = allocator.run(module);

    println!(".section .text");
    println!(".p2align 2"); // align to 4-byte boundary

    for func in module.iter() {
        println!(".globl {}", func.name);
    }

    for func in module.iter() {
        let mut spill_slots = HashMap::<VReg, usize>::new();
        let mut stack_frame: usize = 0;
        let live_intervals = &func_by_intervals.get(&func.name).unwrap();
        for (&vreg, ivs) in live_intervals.iter() {
            if ivs.mark_spilled {
                spill_slots.insert(vreg, stack_frame);
                stack_frame += 8;
            }
        }

        // Prologue
        println!("\n{}:", func.name); // function label
        if stack_frame > 0 {
            println!("  addi sp, sp, -{}", stack_frame);
            // save ra = return address
            println!("  sd ra, {}(sp)", stack_frame - 8);
            // save frame pointer
            println!("  sd s0, {}(sp)", stack_frame - 16);
            println!("  mv s0, sp");
        }

        for block in func.blocks.iter() {
            println!("  .{}:", block.name);

            for instr in block.instrs.iter() {
                // TODO: Add more instructions
                match instr {
                    MachineInstr::Li { rd, imm } => {
                        let phy_reg = to_phys(*rd, live_intervals);
                        println!("  li {}, {}", phy_reg.name(), imm);
                    }

                    MachineInstr::Add { rd, rs1, rs2 } => {
                        let phy_reg = to_phys(*rd, live_intervals);
                        let prs1 = to_phys(*rs1, live_intervals);
                        let prs2 = to_phys(*rs2, live_intervals);

                        println!("  add {}, {}, {}", phy_reg.name(), prs1.name(), prs2.name());
                    }

                    MachineInstr::Mul { rd, rs1, rs2 } => {
                        let phy_reg = to_phys(*rd, live_intervals);
                        let prs1 = to_phys(*rs1, live_intervals);
                        let prs2 = to_phys(*rs2, live_intervals);

                        println!("  mul {}, {}, {}", phy_reg.name(), prs1.name(), prs2.name());
                    }

                    MachineInstr::Sub { rd, rs1, rs2 } => {
                        let phy_reg = to_phys(*rd, live_intervals);
                        let prs1 = to_phys(*rs1, live_intervals);
                        let prs2 = to_phys(*rs2, live_intervals);

                        println!("  sub {}, {}, {}", phy_reg.name(), prs1.name(), prs2.name());
                    }

                    MachineInstr::Div { rd, rs1, rs2 } => {
                        let phy_reg = to_phys(*rd, live_intervals);
                        let prs1 = to_phys(*rs1, live_intervals);
                        let prs2 = to_phys(*rs2, live_intervals);

                        println!("  div {}, {}, {}", phy_reg.name(), prs1.name(), prs2.name());
                    }

                    MachineInstr::Mv { rd, rs1 } => {
                        let phy_reg = to_phys(*rd, live_intervals);
                        let prs1 = to_phys(*rs1, live_intervals);

                        println!("  mv {}, {}", phy_reg.name(), prs1.name());
                    }

                    MachineInstr::Sw { rs1, offset, base } => {
                        let rs = to_phys(*rs1, live_intervals);
                        let base_val = to_phys(*base, live_intervals);

                        println!("  sw {}, {}({})", rs.name(), offset, base_val.name());
                    }

                    MachineInstr::Call { func } => {
                        println!("  call {}", func);
                    }

                    MachineInstr::Jmp { label } => {
                        println!("  j {}", label);
                    }

                    MachineInstr::Jal { rd, label } => {
                        println!("  jal {}, {}", to_phys(*rd, live_intervals).name(), label);
                    }

                    MachineInstr::Beqz { rs1, label } => {
                        //println!("{:#?}", rs1);
                        let rs = to_phys(*rs1, live_intervals);
                        println!("  beqz {}, {}", rs.name(), label);
                    }

                    MachineInstr::Ret { rd } => {
                        if let Some(r) = rd {
                            let phy_reg = to_phys(*r, live_intervals);
                            println!("  ret {}", phy_reg.name());
                        } else {
                            println!("  ret");
                        }
                    }

                    _ => {}
                }
            }
        }

        if stack_frame > 0 {
            // save ra = return address
            println!("  ld s0, {}(sp)", stack_frame - 16);
            println!("  ld ra, {}(sp)", stack_frame - 8);
            // save frame pointer
            println!("  addi sp, sp, {}", stack_frame);
        }
    }
}
