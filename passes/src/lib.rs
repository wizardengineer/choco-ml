pub mod constant_folding;
pub mod constant_propagate;
pub mod deadcode_removal;
pub mod liveness;
pub mod pass_manager;
pub use constant_folding::ConstantFoldPass;
pub use constant_propagate::ConstantPropagationPass;
pub use deadcode_removal::DeadCodeRemovalPass;
pub use liveness::*;
pub use pass_manager::FunctionPass;
pub use pass_manager::PassManager;

// TODO: Need to create a proper test for this crate
#[cfg(test)]
mod tests {
    use super::*;

    use ir::{IrBasicBlock, IrFunction, IrInstruction, SSAFormation};

    /// Build the 5-block “diamond” CFG:
    ///
    ///      0
    ///      │
    ///      1
    ///     / \
    ///    2   3
    ///     \ /
    ///      4
    ///      │
    ///      5
    fn diamond_cfg() -> IrFunction {
        let block_labels = ["entry", "A", "B", "C", "D", "Exit"];

        let preds = vec![
            Vec::new(), // 0: entry
            vec![0],    // 1: A
            vec![1],    // 2: B
            vec![1],    // 3: C
            vec![2, 3], // 4: D (preds are 2 & 3)
            vec![4],    // 5: exit
        ];

        let mut blocks = Vec::new();
        for (i, &label) in block_labels.iter().enumerate() {
            blocks.push(IrBasicBlock {
                label: label.to_string(),
                instrs: Vec::new(),
                preds: preds[i].clone(),
                succs: Vec::new(),
            });
        }

        let mut label_to_idx = std::collections::HashMap::new();
        for (i, &label) in block_labels.iter().enumerate() {
            label_to_idx.insert(label.to_string(), i);
        }

        IrFunction {
            name: "diamond".to_string(),
            args: Vec::new(),
            blocks,
            label_to_idx,
        }
    }

    /// Helper function for creating multiple definitions for further testing
    fn create_def_sites(func: &mut IrFunction) -> anyhow::Result<()> {
        // Set of instrs that we'll be using for definitions sites
        // both block B & C are going to be a definition of var X that will then be managed
        // by block D (maybe)
        let def_x_b = IrInstruction::Assign {
            lhs: "x".to_string(),
            rhs: "5".to_string(),
        };

        let def_x_c = IrInstruction::Assign {
            lhs: "x".to_string(),
            rhs: "10".to_string(),
        };

        // index 2 is block B
        func.blocks[2].instrs.push(def_x_b.clone());

        // index 3 is block C
        func.blocks[3].instrs.push(def_x_c.clone());

        //func.blocks[4].instrs.insert(
        //    0,
        //    IrInstruction::Phi {
        //        dest: "x".to_string(),
        //        sources: vec![None, None],
        //    },
        //);
        //
        Ok(())
    }

    #[test]
    // TODO: Need to finish this
    fn simple_test_liveness() {
        let mut func = diamond_cfg();
        create_def_sites(&mut func).unwrap();
        let mut temp_funcs = vec![func];
        let _ = SSAFormation::new(&mut temp_funcs).unwrap();
        let (live_out, live_in) = compute_liveness(&temp_funcs[0]);
        println!("{:#?}", live_out);
        println!("{:#?}", live_in);

        assert_eq!(4, 4);
    }
}
