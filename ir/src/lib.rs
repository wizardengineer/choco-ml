pub mod cfg;
pub mod ssa;
pub use cfg::IrBasicBlock;
pub use cfg::IrFunction;
pub use cfg::IrInstruction;
pub use cfg::IrModule;
pub use ssa::SSAFormation;

/// Help with having more readable code
pub type BlockID = usize;

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        type_name_of(f)
            .rsplit("::")
            .find(|&part| part != "f" && part != "{{closure}}")
            .expect("Short function name")
    }};
}

#[cfg(test)]
mod tests {
    use crate::cfg::{collect_defs, IrBasicBlock};

    use super::*;

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

    #[test]
    fn test_idom_df_and_domtree_on_diamond() {
        let func = diamond_cfg();

        let mut temp_funcs = vec![func];
        let mut ssa = SSAFormation::new(&mut temp_funcs).unwrap();

        // IDOM Compute
        ssa.compute_idom(&temp_funcs[0]).unwrap();
        println!("Test Function: {}", function!());
        println!("  Idom: {:?}", &ssa.idom);
        assert_eq!(ssa.idom[&0], 0);
        assert_eq!(ssa.idom[&1], 0);
        assert_eq!(ssa.idom[&2], 1);
        assert_eq!(ssa.idom[&3], 1);
        assert_eq!(ssa.idom[&4], 1);
        assert_eq!(ssa.idom[&5], 4);

        ssa.compute_df(&temp_funcs[0]).unwrap();
        let df = &ssa.dom_frontier;
        println!("  DomFrontier: {:?}", &df);
        assert_eq!(df.get(&2).unwrap().clone(), vec![4]);
        assert_eq!(df.get(&3).unwrap().clone(), vec![4]);

        ssa.build_dom_tree().unwrap();

        let dt = &ssa.dom_tree;
        println!("  DomTree: {:?}", dt);
        assert_eq!(dt.get(&4).unwrap().clone(), vec![5]);
        let mut kids = dt.get(&1).unwrap().clone();
        kids.sort();

        assert_eq!(kids, vec![2, 3, 4]);
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
    fn test_collect_defs_of_two_different_defs() {
        let mut func = diamond_cfg();
        create_def_sites(&mut func).unwrap();
        let defs_map = collect_defs(&func);

        println!("Test Function: {}", function!());
        let x_defintion_sites = defs_map.get("x").unwrap();
        println!("  DefintionMap: {:?}", defs_map);
        assert_eq!(x_defintion_sites.len(), 2);
        assert!(x_defintion_sites.contains(&2));
        assert!(x_defintion_sites.contains(&3));
        assert_eq!(defs_map.len(), 1);
    }

    #[test]
    fn test_simple_phi_testing() {
        let mut func = diamond_cfg();
        create_def_sites(&mut func).unwrap();
        let defs_map = collect_defs(&func);
        let mut temp_funcs = vec![func];
        let ssa = SSAFormation::new(&mut temp_funcs).unwrap();

        println!("Test Function: {}", function!());
        //let x_defintion_sites = defs_map.get("x").unwrap();

        println!("  DefintionMap: {:?}", defs_map);

        for block in &temp_funcs[0].blocks {
            for instr in block.instrs.clone() {
                println!("    {:?}", instr);
            }
        }
    }
}
