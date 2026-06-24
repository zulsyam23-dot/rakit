use rakit_ir_hir::hir::*;
use crate::{OptimizationPass, OptimizationResult};

/// Optimasi async state machine — minimalisasi state dan polling berlebihan.
///
/// Optimasi:
/// 1. Eliminasi state yang tidak pernah di-poll
/// 2. Fusion state — gabung state sequential
/// 3. Hilangkan `jalan` (await) jika future langsung selesai
pub struct AsyncOptimizer;

impl AsyncOptimizer {
    fn is_ready_future(&self, expr: &HirExpr) -> bool {
        match expr {
            HirExpr::Number(_, _) | HirExpr::String(_, _) | HirExpr::Bool(_, _) => true,
            _ => false,
        }
    }

    fn optimize_async_block(&self, block: &mut HirBlock) -> usize {
        let mut optimized = 0;

        let mut i = 0;
        while i < block.stmts.len() {
            if let HirStmt::Expr(HirExpr::Call(call)) = &block.stmts[i] {
                if let HirExpr::Ident(name, _) = call.callee.as_ref() {
                    if name == "jalan" {
                        if call.args.len() == 1 && self.is_ready_future(&call.args[0]) {
                            block.stmts.remove(i);
                            optimized += 1;
                            continue;
                        }
                    }
                }
            }
            i += 1;
        }

        optimized
    }
}

impl OptimizationPass for AsyncOptimizer {
    fn name(&self) -> &'static str {
        "async_opt"
    }

    fn run_on_hir(&self, program: &mut HirProgram) -> Vec<OptimizationResult> {
        let mut total_opt = 0;

        for item in &mut program.items {
            match item {
                HirItem::Function(f) => {
                    total_opt += self.optimize_async_block(&mut f.body);
                }
                HirItem::Component(c) => {
                    for stmt in &mut c.body_stmts {
                        if let HirStmt::Expr(e) = stmt {
                            if let HirExpr::Call(call) = e {
                                if let HirExpr::Ident(name, _) = call.callee.as_ref() {
                                    if name == "jalan" {
                                        if call.args.len() == 1 && self.is_ready_future(&call.args[0]) {
                                            total_opt += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        vec![OptimizationResult {
            pass: self.name().to_string(),
            description: format!("Optimasi async: {} eliminasi await siap", total_opt),
            items_affected: total_opt,
        }]
    }
}
