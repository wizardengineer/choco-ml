use ir::{IrBasicBlock, IrFunction};
use std::collections::HashSet;

/// Helps with determining which value or variable is alives through out the function
pub fn compute_liveness(func: &IrFunction) -> (Vec<HashSet<String>>, Vec<HashSet<String>>) {
    let n = func.blocks.len();
    let mut live_out: Vec<HashSet<String>> = vec![HashSet::new(); n];
    let mut live_in: Vec<HashSet<String>> = vec![HashSet::new(); n];
    let mut uses: Vec<HashSet<String>> = vec![HashSet::new(); n];
    let mut defs: Vec<HashSet<String>> = vec![HashSet::new(); n];

    for (i, block) in func.blocks.iter().enumerate() {
        // Compute Use & Def chains for each block
        let (d, u) = compute_block_def_use(block);
        uses[i] = u;
        defs[i] = d;
    }

    // LiveOut Formula: LiveOut[1] = LiveIn[2]
    // LiveIn Formula:  LiveIn[2] = Use[2] ∪ ( LiveOut[2]  / Def[2] )
    //
    // Fix-pointed iteration (backwards)
    loop {
        let mut changed = false;

        for b in (0..n).rev() {
            let old_in = live_in[b].clone();
            let old_out = live_out[b].clone();

            live_out[b].clear();
            for &s in &func.blocks[b].succs {
                live_out[b].extend(live_in[s].iter().cloned());
            }

            // (LiveOut[b] / Def[b])
            let mut differences: HashSet<String> = HashSet::new();
            for var in &live_out[b] {
                if defs[b].contains(var) {
                    continue;
                }
                differences.insert(var.clone());
            }

            live_in[b].clear();
            live_in[b].extend(uses[b].iter().cloned());
            live_in[b].extend(differences.iter().cloned());

            // if we detected any changes
            if old_in != live_in[b] || old_out != live_out[b] {
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    (live_out, live_in)
}

/// Returns the set of defintions & uses for each variable in a block
pub fn compute_block_def_use(block: &IrBasicBlock) -> (HashSet<String>, HashSet<String>) {
    let mut defs = HashSet::new();
    let mut uses = HashSet::new();

    for instr in block.instrs.iter() {
        for def in instr.defs() {
            defs.insert(def.clone());
        }

        // Anything that is used before you define it
        for u in instr.uses() {
            if !defs.contains(&u) {
                uses.insert(u.clone());
            }
        }
    }

    (defs, uses)
}
