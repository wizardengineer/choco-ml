use crate::liveness::compute_liveness;
use crate::pass_manager::FunctionPass;
use ir::IrFunction;
use ir::IrInstruction;
use std::collections::HashSet;

/// Intraprocedural Constant Propagation
pub struct DeadCodeRemovalPass {}

impl FunctionPass for DeadCodeRemovalPass {
    fn name(&self) -> &str {
        "DeadCodeRemovalPass"
    }

    fn run_on_function(&mut self, function: &mut IrFunction) -> bool {
        eliminate_deadcode(function);
        true
    }
}

fn eliminate_deadcode(func: &mut IrFunction) {
    // iterate over each block then for each block,
    // iterate over them in reverse

    let (live_out, _live_in) = compute_liveness(func);

    for (b, block) in func.blocks.iter_mut().enumerate() {
        let mut live: HashSet<String> = live_out[b].clone();

        let mut new_instrs: Vec<IrInstruction> = Vec::with_capacity(block.instrs.len());
        for instr in block.instrs.iter().rev() {
            // check to see if a definition is live
            if let Some(d) = instr.defs().first() {
                // if not live, then skip
                if !live.contains(d) {
                    continue;
                }
                // in case we had kept a old definition, we want to remove
                live.remove(d);
            }

            for u in instr.uses() {
                live.insert(u.clone());
            }

            new_instrs.push(instr.clone());
        }
        new_instrs.reverse();
        block.instrs = new_instrs;
    }
}
