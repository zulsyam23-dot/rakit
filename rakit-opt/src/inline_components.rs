use std::collections::HashSet;
use rakit_ir_hir::hir::*;
use crate::{OptimizationPass, OptimizationResult};

/// Inline komponen kecil yang tidak punya hooks dan render tree sederhana.
pub struct InlineComponents;

impl InlineComponents {
    fn can_inline(&self, component: &HirComponent) -> bool {
        let has_hooks = !component.hook_calls.is_empty();
        let simple_body = component.body_stmts.len() <= 2;
        let simple_render = self.is_simple_render(&component.render);
        !has_hooks && simple_body && simple_render
    }

    fn is_simple_render(&self, expr: &HirExpr) -> bool {
        match expr {
            HirExpr::JsxElement(elem) => {
                let depth = self.max_depth(elem, 0);
                depth <= 3
            }
            _ => false,
        }
    }

    fn max_depth(&self, elem: &HirJsxElement, current: usize) -> usize {
        let mut max = current + 1;
        for child in &elem.children {
            if let HirExpr::JsxElement(child_elem) = child {
                let d = self.max_depth(child_elem, current + 1);
                max = max.max(d);
            }
        }
        max
    }

    fn collect_inlineable(&self, program: &HirProgram) -> HashSet<String> {
        program.items
            .iter()
            .filter_map(|item| {
                if let HirItem::Component(c) = item {
                    if self.can_inline(c) {
                        Some(c.name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn inline_in_expr(
        &self,
        expr: &mut HirExpr,
        inlineable: &HashSet<String>,
        results: &mut Vec<OptimizationResult>,
    ) {
        match expr {
            HirExpr::JsxElement(elem) => {
                for (_, val) in &mut elem.attrs {
                    self.inline_in_expr(val, inlineable, results);
                }
                for child in &mut elem.children {
                    self.inline_in_expr(child, inlineable, results);
                }
            }
            HirExpr::Call(call) => {
                if let HirExpr::Ident(name, _) = call.callee.as_ref() {
                    if inlineable.contains(name.as_str()) {
                        results.push(OptimizationResult {
                            pass: "inline_components".into(),
                            description: format!("Inline komponen '{}'", name),
                            items_affected: 1,
                        });
                    }
                }
                for arg in &mut call.args {
                    self.inline_in_expr(arg, inlineable, results);
                }
                self.inline_in_expr(&mut call.callee, inlineable, results);
            }
            HirExpr::Binary(bin) => {
                self.inline_in_expr(&mut bin.lhs, inlineable, results);
                self.inline_in_expr(&mut bin.rhs, inlineable, results);
            }
            HirExpr::Unary(u) => {
                self.inline_in_expr(&mut u.expr, inlineable, results);
            }
            HirExpr::Block(block) => {
                for stmt in &mut block.stmts {
                    match stmt {
                        HirStmt::Expr(e) => self.inline_in_expr(e, inlineable, results),
                        HirStmt::Let(l) => self.inline_in_expr(&mut l.value, inlineable, results),
                        HirStmt::Return(Some(e)) => self.inline_in_expr(e, inlineable, results),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl OptimizationPass for InlineComponents {
    fn name(&self) -> &'static str {
        "inline_components"
    }

    fn run_on_hir(&self, program: &mut HirProgram) -> Vec<OptimizationResult> {
        let mut results = Vec::new();
        let inlineable = self.collect_inlineable(program);

        if inlineable.is_empty() {
            return results;
        }

        for item in &mut program.items {
            match item {
                HirItem::Function(f) => {
                    let mut block_expr = HirExpr::Block(f.body.clone());
                    self.inline_in_expr(&mut block_expr, &inlineable, &mut results);
                    if let HirExpr::Block(b) = block_expr {
                        f.body = b;
                    }
                }
                HirItem::Component(c) => {
                    for stmt in &mut c.body_stmts {
                        match stmt {
                            HirStmt::Expr(e) => self.inline_in_expr(e, &inlineable, &mut results),
                            HirStmt::Let(l) => self.inline_in_expr(&mut l.value, &inlineable, &mut results),
                            _ => {}
                        }
                    }
                    self.inline_in_expr(&mut c.render, &inlineable, &mut results);
                }
                _ => {}
            }
        }

        results
    }
}
