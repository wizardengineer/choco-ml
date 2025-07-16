use ir::IrFunction;
use ir::IrModule;

/// This trait will be inherited by optimizations or transformations of
/// on functions within the Module scope
pub trait FunctionPass {
    fn name(&self) -> &str;

    fn run_on_function(&mut self, function: &mut IrFunction) -> bool;
}

#[derive(Default)]
pub struct PassManager {
    passes: Vec<Box<dyn FunctionPass>>,
}

impl PassManager {
    pub fn new() -> PassManager {
        PassManager { passes: Vec::new() }
    }

    pub fn run(&mut self, module: &mut IrModule) {
        // loop throught each function in the module and run the pass
        for func in module.functions.iter_mut() {
            // loop there each of the element in the passes vector
            for pass in self.passes.iter_mut() {
                let changed = pass.run_on_function(func);
                if !changed {
                    // TODO: find a better way of dealing with this
                    // maybe add an erroring system?
                    break;
                }
            }
        }
    }

    pub fn add_pass<P: FunctionPass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }
}
