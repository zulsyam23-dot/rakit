use rakit_ir_hir::hir::*;
use crate::{OptimizationPass, OptimizationResult};

/// Constant propagation — evaluasi ekspresi konstan pada HIR.
///
/// Transformasi:
///   konstan x = 5 + 3  →  konstan x = 8
///   <div className={"btn-" + "primary"}>  →  <div className="btn-primary">
pub struct ConstantPropagation;

impl ConstantPropagation {
    fn fold_constants(&self, expr: &mut HirExpr) -> bool {
        match expr {
            HirExpr::Number(_, _) | HirExpr::String(_, _) | HirExpr::Bool(_, _) | HirExpr::Null(_) => {
                false
            }
            HirExpr::Binary(bin) => {
                let lhs_const = self.fold_constants(&mut bin.lhs);
                let rhs_const = self.fold_constants(&mut bin.rhs);

                if let (HirExpr::Number(lv, _), HirExpr::Number(rv, _)) = (bin.lhs.as_ref(), bin.rhs.as_ref()) {
                    let result = match bin.op {
                        HirBinaryOp::Add => Some(*lv + *rv),
                        HirBinaryOp::Sub => Some(*lv - *rv),
                        HirBinaryOp::Mul => Some(*lv * *rv),
                        HirBinaryOp::Div => {
                            if *rv != 0.0 { Some(*lv / *rv) } else { None }
                        }
                        HirBinaryOp::Mod => {
                            if *rv != 0.0 { Some(*lv % *rv) } else { None }
                        }
                        _ => None,
                    };

                    if let Some(val) = result {
                        *expr = HirExpr::Number(val, bin.ty.clone());
                        return true;
                    }
                }

                if let (HirExpr::String(ls, _), HirExpr::String(rs, _)) = (bin.lhs.as_ref(), bin.rhs.as_ref()) {
                    if matches!(bin.op, HirBinaryOp::Concat) {
                        let result = format!("{}{}", ls, rs);
                        *expr = HirExpr::String(result, bin.ty.clone());
                        return true;
                    }
                }

                lhs_const || rhs_const
            }
            HirExpr::Unary(u) => {
                let inner_const = self.fold_constants(&mut u.expr);
                if let HirExpr::Number(val, _) = u.expr.as_ref() {
                    match u.op {
                        HirUnaryOp::Neg => {
                            *expr = HirExpr::Number(-val, u.ty.clone());
                            return true;
                        }
                        _ => {}
                    }
                }
                inner_const
            }
            HirExpr::Block(block) => {
                let mut changed = false;
                for stmt in &mut block.stmts {
                    if let HirStmt::Let(l) = stmt {
                        if self.fold_constants(&mut l.value) {
                            changed = true;
                        }
                    }
                }
                changed
            }
            HirExpr::JsxElement(elem) => {
                let mut changed = false;
                for (_, val) in &mut elem.attrs {
                    if self.fold_constants(val) {
                        changed = true;
                    }
                }
                for child in &mut elem.children {
                    if self.fold_constants(child) {
                        changed = true;
                    }
                }
                changed
            }
            HirExpr::Call(call) => {
                let mut changed = false;
                for arg in &mut call.args {
                    if self.fold_constants(arg) {
                        changed = true;
                    }
                }
                changed
            }
            _ => false,
        }
    }
}

impl OptimizationPass for ConstantPropagation {
    fn name(&self) -> &'static str {
        "constant_props"
    }

    fn run_on_hir(&self, program: &mut HirProgram) -> Vec<OptimizationResult> {
        let mut total_folded = 0;

        for item in &mut program.items {
            match item {
                HirItem::Function(f) => {
                    for stmt in &mut f.body.stmts {
                        if let HirStmt::Let(l) = stmt {
                            if self.fold_constants(&mut l.value) {
                                total_folded += 1;
                            }
                        }
                    }
                }
                HirItem::Component(c) => {
                    for stmt in &mut c.body_stmts {
                        if let HirStmt::Let(l) = stmt {
                            if self.fold_constants(&mut l.value) {
                                total_folded += 1;
                            }
                        }
                    }
                    if self.fold_constants(&mut c.render) {
                        total_folded += 1;
                    }
                }
                _ => {}
            }
        }

        vec![OptimizationResult {
            pass: self.name().to_string(),
            description: format!("Konstanta folding: {} ekspresi", total_folded),
            items_affected: total_folded,
        }]
    }
}
