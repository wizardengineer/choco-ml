use crate::pass_manager::FunctionPass;
use ir::IrFunction;
use ir::IrInstruction;
use ir::cfg::Literal;
use std::collections::HashMap;

/// Intraprocedural Constant Propagation
pub struct ConstantPropagationPass {}

impl FunctionPass for ConstantPropagationPass {
    fn name(&self) -> &str {
        "ConstantPropagationPass"
    }

    fn run_on_function(&mut self, function: &mut IrFunction) -> bool {
        let mut const_env: HashMap<String, Option<Literal>> = HashMap::new();
        for blocks in function.blocks.iter_mut() {
            for instr in blocks.instrs.iter_mut() {
                match instr {
                    // TODO: Need to add more patterns to match for
                    IrInstruction::Const { dest, value } => {
                        const_env.insert(dest.clone(), Some(value.clone()));
                    }

                    IrInstruction::Assign { rhs, .. } => {
                        if let Some(Literal::Int(j)) = const_env.get(rhs).cloned().flatten() {
                            *rhs = j.to_string();
                        }
                    }

                    IrInstruction::Add { lhs, rhs, .. }
                    | IrInstruction::Mul { lhs, rhs, .. }
                    | IrInstruction::Sub { lhs, rhs, .. }
                    | IrInstruction::Div { lhs, rhs, .. } => {
                        if let Some(Literal::Int(i)) = const_env.get(lhs).cloned().flatten() {
                            *lhs = i.to_string();
                        }

                        if let Some(Literal::Int(j)) = const_env.get(rhs).cloned().flatten() {
                            *rhs = j.to_string();
                        }
                    }

                    IrInstruction::Eq { lhs, rhs, .. }
                    | IrInstruction::Lt { lhs, rhs, .. }
                    | IrInstruction::Gt { lhs, rhs, .. }
                    | IrInstruction::Ge { lhs, rhs, .. } => {
                        if let Some(Literal::Bool(i)) = const_env.get(lhs).cloned().flatten() {
                            *lhs = i.to_string();
                        }

                        if let Some(Literal::Bool(j)) = const_env.get(rhs).cloned().flatten() {
                            *rhs = j.to_string();
                        }
                    }

                    IrInstruction::Br { cond, .. } => {
                        if let Some(Literal::Bool(j)) = const_env.get(cond).cloned().flatten() {
                            *cond = j.to_string();
                        }
                    }

                    IrInstruction::Ret { args } => {
                        for arg in args.iter_mut() {
                            if let Some(Literal::Int(i)) = const_env.get(arg).cloned().flatten() {
                                *arg = i.to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        true
    }
}
