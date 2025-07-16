use crate::pass_manager::FunctionPass;
use ir::cfg::Literal;
use ir::IrFunction;
use ir::IrInstruction;

/// Intraprocedural Constant Fold
pub struct ConstantFoldPass {}

impl FunctionPass for ConstantFoldPass {
    fn name(&self) -> &str {
        "ConstantFoldPass"
    }

    fn run_on_function(&mut self, function: &mut IrFunction) -> bool {
        for blocks in function.blocks.iter_mut() {
            for instr in blocks.instrs.iter_mut() {
                // TODO: Added more folds
                match instr {
                    IrInstruction::Add { dest, lhs, rhs } => {
                        if rhs.parse::<i64>().is_err() || lhs.parse::<i64>().is_err() {
                            continue;
                        }

                        let right = rhs.parse::<i64>().unwrap();
                        let left = lhs.parse::<i64>().unwrap();
                        let sum = left + right;
                        *instr = IrInstruction::Const {
                            dest: dest.to_string(),
                            value: Literal::Int(sum),
                        };
                    }

                    IrInstruction::Mul { dest, lhs, rhs } => {
                        if rhs.parse::<i64>().is_err() || lhs.parse::<i64>().is_err() {
                            continue;
                        }

                        let right = rhs.parse::<i64>().unwrap();
                        let left = lhs.parse::<i64>().unwrap();
                        let product = left * right;
                        *instr = IrInstruction::Const {
                            dest: dest.to_string(),
                            value: Literal::Int(product),
                        };
                    }
                    _ => {}
                }
            }
        }
        true
    }
}
