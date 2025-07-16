use crate::BlockID;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub args: Vec<String>,
    pub blocks: Vec<IrBasicBlock>,
    pub label_to_idx: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct IrBasicBlock {
    pub label: String,
    pub instrs: Vec<IrInstruction>,
    pub preds: Vec<usize>,
    pub succs: Vec<usize>,
}

impl IrFunction {
    // just in case i was to do some testing
    pub fn new(func_name: &str) -> Self {
        Self {
            name: func_name.to_string(),
            args: Vec::new(),
            blocks: Vec::new(),
            label_to_idx: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, label: &str) -> usize {
        // current block we're on
        let idx = self.blocks.len();

        self.blocks.push(IrBasicBlock {
            label: label.to_string(),
            instrs: Vec::new(),
            preds: Vec::new(),
            succs: Vec::new(),
        });

        // build our label to index mapping, for each
        // block we add to the Block vectors
        self.label_to_idx.insert(label.to_string(), idx);

        // return index of newly added block index
        idx
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.blocks[from].succs.push(to);
        self.blocks[to].preds.push(from);
    }

    pub fn append_instr(&mut self, idx: usize, instr: &IrInstruction) {
        self.blocks[idx].instrs.push(instr.clone());
    }

    pub fn block_index(&self, label: &String) -> Option<usize> {
        self.label_to_idx.get(label).copied()
    }
}

#[derive(Debug, Clone)]
pub enum IrInstruction {
    // == Arithematic ==
    Add {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Mul {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Sub {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Div {
        dest: String,
        lhs: String,
        rhs: String,
    },

    // == Comparsion ==
    Eq {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Lt {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Gt {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Ge {
        dest: String,
        lhs: String,
        rhs: String,
    },

    Le {
        dest: String,
        lhs: String,
        rhs: String,
    },

    // == Logical Operator ==
    Not {
        dest: String,
        args: String,
    },

    Or {
        dest: String,
        lhs: String,
        rhs: String,
    },

    And {
        dest: String,
        lhs: String,
        rhs: String,
    },

    // == Control Flow ==
    Call {
        target_func: String,
        args: Vec<String>,
        dest: Option<String>,
    },

    Br {
        cond: String,
        then_lbl: String,
        else_lbl: String,
    },

    Jmp {
        label: String,
    },

    Ret {
        args: Vec<String>,
    },

    Phi {
        dest: String,                 // value the be dictated by previous values
        sources: Vec<Option<String>>, // this will store the blocks id of preds for blocks
    },

    // == Literals ==
    Const {
        dest: String,
        value: Literal,
    },

    // == Misc ==
    Print {
        values: Vec<String>,
    },

    Assign {
        lhs: String,
        rhs: String,
    },
}

impl IrInstruction {
    // Returns a slice of a defined variable
    // describes what name does this instruction *write*
    pub fn defs(&self) -> &[String] {
        match self {
            IrInstruction::Add { dest, .. }
            | IrInstruction::Sub { dest, .. }
            | IrInstruction::Mul { dest, .. }
            | IrInstruction::Div { dest, .. }
            | IrInstruction::Eq { dest, .. }
            | IrInstruction::Lt { dest, .. }
            | IrInstruction::Gt { dest, .. }
            | IrInstruction::Le { dest, .. }
            | IrInstruction::Ge { dest, .. }
            | IrInstruction::Or { dest, .. }
            | IrInstruction::And { dest, .. }
            | IrInstruction::Not { dest, .. }
            | IrInstruction::Const { dest, .. }
            // TODO: Maybe we should remove the assign?
            // Find something else to use
            | IrInstruction::Assign { lhs: dest, .. }
            | IrInstruction::Phi { dest, .. } => std::slice::from_ref(dest),

            IrInstruction::Call { dest, .. } => {
                if let Some(d) = dest {
                    std::slice::from_ref(d)
                } else {
                    &[]
                }
            },

            _ => &[],
        }
    }

    // describes what name does this instruction *reads*
    pub fn uses(&self) -> Vec<String> {
        match self {
            IrInstruction::Add { lhs, rhs, .. }
            | IrInstruction::Sub { lhs, rhs, .. }
            | IrInstruction::Mul { lhs, rhs, .. }
            | IrInstruction::Div { lhs, rhs, .. }
            | IrInstruction::Eq { lhs, rhs, .. }
            | IrInstruction::Lt { lhs, rhs, .. }
            | IrInstruction::Gt { lhs, rhs, .. }
            | IrInstruction::Ge { lhs, rhs, .. }
            | IrInstruction::Le { lhs, rhs, .. }
            | IrInstruction::Or { lhs, rhs, .. }
            | IrInstruction::And { lhs, rhs, .. } => vec![lhs.to_string(), rhs.to_string()],

            IrInstruction::Not { args, .. } => vec![args.to_string()],

            IrInstruction::Br { cond, .. } => vec![cond.to_string()],
            IrInstruction::Call { args, .. } => args.to_vec(),
            IrInstruction::Ret { args, .. } => args.to_vec(),
            IrInstruction::Phi { sources, .. } => sources.iter().flatten().cloned().collect(),

            IrInstruction::Print { values, .. } => values.to_vec(),
            _ => Vec::new(),
        }
    }
}

/// For getting the mapping of each variable block(s) where variable might be defined
pub fn collect_defs(func: &IrFunction) -> HashMap<String, Vec<BlockID>> {
    let mut defs_map: HashMap<String, Vec<usize>> = HashMap::new();

    for (block_idx, block) in func.blocks.iter().enumerate() {
        for instr in &block.instrs {
            for var in instr.defs() {
                defs_map.entry(var.clone()).or_default().push(block_idx);
            }
        }
    }

    defs_map
}

//TODO: Need to fix this for working with our frontend
struct TmpTodo {}
/// Converting Flat Functions into CFG
fn convert_to_cfg(func: &TmpTodo) -> Result<IrFunction> {
    let mut ir_func = IrFunction::new(&"todo");
    split_into_blocks(&mut ir_func)?;

    wire_block_edges(&mut ir_func)?;

    Ok(ir_func)
}

/// This functions deals with converting the IR into true
/// Control-Flow Graphs by wiring up the blocks
fn wire_block_edges(func: &mut IrFunction) -> Result<()> {
    // Build up the list of Successors & Predecessors fork
    for curr_block_idx in 0..func.blocks.len() {
        if let Some(terminator) = func.blocks[curr_block_idx].instrs.last() {
            match terminator {
                IrInstruction::Br {
                    then_lbl, else_lbl, ..
                } => {
                    let then_idx = func.block_index(then_lbl).unwrap();
                    let else_idx = func.block_index(else_lbl).unwrap();

                    func.add_edge(curr_block_idx, then_idx);
                    func.add_edge(curr_block_idx, else_idx);
                }

                IrInstruction::Jmp { label } => {
                    let target_idx = func.block_index(label).unwrap();
                    func.add_edge(curr_block_idx, target_idx);
                }

                // TODO: I think I'll need to manage this later on?
                IrInstruction::Ret { .. } => {}

                // Fall through the next label, if needed so
                _ => {
                    // check to see if we're still within the range of the blocks list
                    if curr_block_idx + 1 < func.blocks.len() - 1 {
                        func.add_edge(curr_block_idx, curr_block_idx + 1);
                    }
                }
            }
        }
    }

    Ok(())
}

// TODO: Need to finish this
fn split_into_blocks(func: &mut IrFunction) -> Result<()> {
    todo!();
}
