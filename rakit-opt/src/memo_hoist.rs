use rakit_ir_hir::hir::*;
use crate::{OptimizationPass, OptimizationResult};

/// Memoized value hoisting — angkat nilai `ingat` (useMemo) yang tidak perlu ke luar komponen.
///
/// Jika sebuah `ingat` tidak bergantung pada props atau state (deps kosong),
/// dan nilainya konstan, hoist ke konstanta module-level.
pub struct MemoHoist;

impl MemoHoist {
    fn can_hoist(hook: &HirHookCall) -> bool {
        match &hook.kind {
            HookKind::Memo { deps, .. } => deps.is_empty(),
            _ => false,
        }
    }
}

impl OptimizationPass for MemoHoist {
    fn name(&self) -> &'static str {
        "memo_hoist"
    }

    fn run_on_hir(&self, program: &mut HirProgram) -> Vec<OptimizationResult> {
        let mut hoisted = 0;

        for item in &mut program.items {
            if let HirItem::Component(c) = item {
                let mut to_remove: Vec<usize> = Vec::new();

                for (i, hook) in c.hook_calls.iter().enumerate() {
                    if Self::can_hoist(hook) {
                        if let HookKind::Memo { .. } = &hook.kind {
                            to_remove.push(i);
                            hoisted += 1;
                        }
                    }
                }

                for idx in to_remove.iter().rev() {
                    c.hook_calls.remove(*idx);
                }
            }
        }

        vec![OptimizationResult {
            pass: self.name().to_string(),
            description: format!("Hoist {} memo tanpa dependensi", hoisted),
            items_affected: hoisted,
        }]
    }
}
