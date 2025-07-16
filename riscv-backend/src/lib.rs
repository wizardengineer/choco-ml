pub mod instruction_sel;
pub mod machine_ir;
pub mod register_alloc;
pub mod riscv_emission;

pub use instruction_sel::select_instructions;
//pub use machine_ir::MachineBlock;
//pub use machine_ir::MachineFunc;
//pub use machine_ir::MachineInstr;
pub use machine_ir::*;
pub use register_alloc::*;
pub use riscv_emission::emit_riscv;

// some change
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
