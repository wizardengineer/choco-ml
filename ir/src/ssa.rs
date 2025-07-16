use crate::cfg::collect_defs;
use crate::cfg::IrFunction;
use crate::cfg::IrModule;
use crate::BlockID;
use crate::IrInstruction;
use anyhow::Result;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Set up the Dominator Trees and Dominance Frontier
/// Using the Cytron algo for creating a SSA
///
/// Cytron et al.’s SSA-construction recipe bundles all of this into a single,
/// reasonably efficient flow:
///
///1.Compute dominators (often via a faster “Lengauer–Tarjan” or equivalent algorithm,
///   not the naive Braun iteration).
///2.Build the immediate-dominator (idom) tree.
///3.Compute each node’s DF (in a single pass over the CFG + dom-tree).
///4.Place ϕ-nodes for each variable at all blocks in the union of DF(definition blocks).
#[derive(Debug, Default)]
pub struct SSAFormation {
    pub idom: HashMap<BlockID, BlockID>,
    pub dom_tree: HashMap<BlockID, Vec<BlockID>>,
    pub dom_frontier: BTreeMap<BlockID, Vec<BlockID>>,
}

/// Convert our IrModule into a true SSA form
impl TryFrom<&mut IrModule> for SSAFormation {
    type Error = anyhow::Error;

    fn try_from(module: &mut IrModule) -> Result<SSAFormation> {
        let out = SSAFormation::new(&mut module.functions)?;
        Ok(out)
    }
}

impl SSAFormation {
    pub fn new(funcs: &mut [IrFunction]) -> Result<Self> {
        let mut out = SSAFormation::default();

        for func in funcs {
            out.compute_idom(func)?;
            out.compute_df(func)?;
            out.build_dom_tree()?;

            let def_sites_map = collect_defs(func);
            out.phi_insert(func, &def_sites_map);

            let mut counter: HashMap<String, BlockID> = HashMap::new();
            let mut stacks: HashMap<String, Vec<String>> = HashMap::new();

            for (var, _def_sites) in def_sites_map {
                counter.insert(var.clone(), 0);
                stacks.insert(var.clone(), Vec::new());
            }
            rename_pass(0, &out.dom_tree, func, &mut counter, &mut stacks);
        }

        Ok(out)
    }

    // TODO: Later in the future implement lengauer_tarjan_idom
    pub fn compute_idom(&mut self, func: &IrFunction) -> Result<()> {
        let n = func.blocks.len();
        // usize::MAX means the idom is an unknown for now
        let mut idom_vec = vec![usize::MAX; n];

        // entry point to entry
        idom_vec[0] = 0;

        // find the fix-point of the loop
        loop {
            let mut changed = false;
            // b_idx = block index
            // starting from block 1 because idom[0] is 0
            for b in 1..n {
                let preds = &func.blocks[b].preds;

                // Skip for if preds empty, we care for the preds because of the idom
                if preds.is_empty() {
                    continue;
                }

                let mut new_idom = match preds.iter().find(|&&p| idom_vec[p] != usize::MAX) {
                    Some(&p) => p,
                    None => continue,
                };

                // collect into a Vec<usize>
                let others: Vec<usize> = preds
                    .iter()
                    .copied()
                    .filter(|&p| p != new_idom && idom_vec[p] != usize::MAX)
                    .collect();

                // climb the preds in order to see if the dominance chains match
                for p in others {
                    let mut finger1 = p;
                    let mut finger2 = new_idom;
                    while finger1 != finger2 {
                        while finger1 > finger2 {
                            finger1 = idom_vec[finger1];
                        }
                        while finger2 > finger1 {
                            finger2 = idom_vec[finger2];
                        }
                    }
                    new_idom = finger1;
                }

                if idom_vec[b] != new_idom {
                    idom_vec[b] = new_idom;
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        self.idom.clear();
        for (block, &dom) in idom_vec.iter().enumerate() {
            if dom == usize::MAX {
                panic!("could not compute idom for Block {}", block);
            }
            self.idom.insert(block, dom);
        }

        Ok(())
    }

    // TODO: Finish this and dom tree too. Then test it out
    pub fn compute_df(&mut self, func: &IrFunction) -> Result<()> {
        self.dom_frontier.clear();

        for block in &func.blocks {
            let b = func.block_index(&block.label).unwrap();

            // making sure it's a joint point
            if block.preds.len() < 2 {
                continue;
            }

            let idom_b = *self.idom.get(&b).expect("idom wasn't computed");

            for &p in &block.preds {
                let mut runner = p;

                while runner != idom_b {
                    let entry = self.dom_frontier.entry(runner).or_default();
                    if !entry.contains(&b) {
                        entry.push(b);
                    }

                    // climbing up the pred, the one runner is equal to
                    runner = *self.idom.get(&runner).unwrap();
                }
            }
        }

        Ok(())
    }

    pub fn build_dom_tree(&mut self) -> Result<()> {
        self.dom_tree.clear();

        for (&b, &p) in &self.idom {
            // make sure we've skipped the entry
            if b != p {
                self.dom_tree.entry(p).or_default().push(b);
            }
        }
        Ok(())
    }

    pub fn phi_insert(&self, func: &mut IrFunction, def_sites_map: &HashMap<String, Vec<BlockID>>) {
        for (var, blocks_with_defs) in def_sites_map {
            // `var` - the Variable we're looking for
            // `blocks_with_defs` - blocks where `var` is defined at
            let mut worklist: Vec<BlockID> = blocks_with_defs.clone();
            let mut has_phi: HashSet<BlockID> = blocks_with_defs.iter().cloned().collect();

            while let Some(block_id_def) = worklist.pop() {
                if let Some(frontier) = self.dom_frontier.get(&block_id_def) {
                    for &m in frontier {
                        if has_phi.insert(m) {
                            let block = &mut func.blocks[m];
                            block.instrs.insert(
                                0,
                                IrInstruction::Phi {
                                    dest: var.clone(),
                                    sources: vec![None; block.preds.len()],
                                },
                            );

                            worklist.push(m);
                        }
                    }
                }
            }
        }
    }
}

/// Rename pass for all the blocks, it'll convert every indiviual variables in each block
/// with it's own unique name
pub fn rename_pass(
    block_id: BlockID,
    dom_tree: &HashMap<BlockID, Vec<BlockID>>,
    func: &mut IrFunction,
    counter: &mut HashMap<String, BlockID>,
    stacks: &mut HashMap<String, Vec<String>>,
) {
    {
        let blocks = &mut func.blocks;
        // Manage all the Phi-nodes block
        for instr in blocks[block_id].instrs.iter_mut() {
            if let IrInstruction::Phi { dest, .. } = instr {
                *dest = create_new_name(dest, counter, stacks);
            }
        }
        // Rename all non-phi instructions for current block
        for instr in blocks[block_id].instrs.iter_mut() {
            // TODO: Maybe find a better way of handling this? This relates
            // to the ID opcode for Bril...
            match instr {
                IrInstruction::Assign { lhs, rhs } => {
                    *rhs = current_name(rhs, stacks);
                    *lhs = create_new_name(lhs, counter, stacks);
                }

                IrInstruction::Not { dest, args } => {
                    *args = current_name(args, stacks);
                    *dest = create_new_name(dest, counter, stacks);
                }

                // TODO: Added more instructions
                IrInstruction::Add { lhs, rhs, dest }
                | IrInstruction::Mul { lhs, rhs, dest }
                | IrInstruction::Sub { lhs, rhs, dest }
                | IrInstruction::Div { lhs, rhs, dest }
                | IrInstruction::Eq { lhs, rhs, dest }
                | IrInstruction::Lt { lhs, rhs, dest }
                | IrInstruction::Gt { lhs, rhs, dest }
                | IrInstruction::Ge { lhs, rhs, dest }
                | IrInstruction::Le { lhs, rhs, dest }
                | IrInstruction::Or { lhs, rhs, dest }
                | IrInstruction::And { lhs, rhs, dest } => {
                    *lhs = current_name(lhs, stacks);
                    *rhs = current_name(rhs, stacks);
                    *dest = create_new_name(dest, counter, stacks);
                }

                IrInstruction::Call { args, dest, .. } => {
                    if !args.is_empty() {
                        for a in args.iter_mut() {
                            *a = current_name(a, stacks);
                        }
                    }

                    if let Some(d) = dest {
                        *dest = Some(create_new_name(d, counter, stacks));
                    }
                }

                IrInstruction::Print { values } => {
                    if !values.is_empty() {
                        for a in values.iter_mut() {
                            *a = current_name(a, stacks);
                        }
                    }
                }

                IrInstruction::Ret { args } => {
                    if !args.is_empty() {
                        for a in args.iter_mut() {
                            *a = current_name(a, stacks);
                        }
                    }
                }

                _ => {}
            }
        }
    }

    // Check each of the successors of the current Block and fill in the Phi-nodes
    // if needed
    for succ in func.blocks[block_id].succs.clone() {
        let succ_block = &mut func.blocks[succ];
        for instr in succ_block.instrs.iter_mut() {
            if let IrInstruction::Phi { dest, sources } = instr {
                let idx = succ_block
                    .preds
                    .iter()
                    .position(|&p| p == block_id)
                    .unwrap();
                // Source is the size of the preds
                sources[idx] = Some(current_name(dest, stacks));
            }
        }
    }

    // Recursively rename each immediate child of a block through the dominator tree
    if let Some(child_blocks) = dom_tree.get(&block_id) {
        for &child in child_blocks {
            rename_pass(child, dom_tree, func, counter, stacks);
        }
    }

    // Now we have to pop all the values on the SSA rename stacks hashmap
    // in order to have a distinct values
    for instr in &func.blocks[block_id].instrs {
        //println!("Instr!! {:#?}", &instr);
        //println!("Block!! {:#?}", &func.blocks[block_id]);
        for var in instr.defs() {
            //println!("Var!! {}", var);
            if stacks.contains_key(var) {
                stacks.get_mut(var).expect("Something is wrong twin").pop();
            }
        }
    }
}

/// Helper function with getting the current variable with subscript (if there is any) on the stack
fn current_name(var: &String, stacks: &HashMap<String, Vec<String>>) -> String {
    stacks
        .get(var)
        .and_then(|stk| stk.last().cloned())
        .unwrap_or_else(|| var.to_string())
}

/// Helper function for creating a new name for variables in SSA Form
fn create_new_name(
    var: &str,
    counter: &mut HashMap<String, BlockID>,
    stacks: &mut HashMap<String, Vec<String>>,
) -> String {
    let count = counter.entry(var.to_string()).or_insert(0);
    *count += 1;

    let new_var = format!("{}${}", &var, count);
    stacks
        .entry(var.to_string())
        .or_default()
        .push(new_var.clone());
    new_var
}
