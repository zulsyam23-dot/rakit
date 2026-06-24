pub mod inline_components;
pub mod dce_vdom;
pub mod constant_props;
pub mod tree_shaking;
pub mod memo_hoist;
pub mod async_opt;

use rakit_ir_hir::hir::HirProgram;

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub pass: String,
    pub description: String,
    pub items_affected: usize,
}

pub trait OptimizationPass {
    fn name(&self) -> &'static str;
    fn run_on_hir(&self, program: &mut HirProgram) -> Vec<OptimizationResult>;
}

pub struct Optimizer {
    passes: Vec<Box<dyn OptimizationPass>>,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer { passes: Vec::new() }
    }

    pub fn with_default_passes() -> Self {
        let mut opt = Optimizer::new();
        opt.add_pass(Box::new(inline_components::InlineComponents));
        opt.add_pass(Box::new(dce_vdom::DceVdom));
        opt.add_pass(Box::new(constant_props::ConstantPropagation));
        opt.add_pass(Box::new(tree_shaking::TreeShaking));
        opt.add_pass(Box::new(memo_hoist::MemoHoist));
        opt.add_pass(Box::new(async_opt::AsyncOptimizer));
        opt
    }

    pub fn add_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        self.passes.push(pass);
    }

    pub fn run(&self, program: &mut HirProgram) -> Vec<OptimizationResult> {
        let mut all_results = Vec::new();
        for pass in &self.passes {
            let results = pass.run_on_hir(program);
            all_results.extend(results);
        }
        all_results
    }

    pub fn run_optimized(&self, program: &mut HirProgram) -> OptimizationSummary {
        let results = self.run(program);
        let mut total_affected = 0;
        for r in &results {
            total_affected += r.items_affected;
        }
        OptimizationSummary {
            passes_run: self.passes.len(),
            total_items_affected: total_affected,
            results,
        }
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::with_default_passes()
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationSummary {
    pub passes_run: usize,
    pub total_items_affected: usize,
    pub results: Vec<OptimizationResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creates_with_default_passes() {
        let opt = Optimizer::with_default_passes();
        assert_eq!(opt.passes.len(), 6);
    }

    #[test]
    fn test_optimizer_empty_program() {
        let opt = Optimizer::with_default_passes();
        let mut program = HirProgram { items: vec![] };
        let summary = opt.run_optimized(&mut program);
        assert_eq!(summary.passes_run, 6);
    }
}
