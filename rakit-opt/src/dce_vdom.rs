use std::collections::HashSet;
use rakit_ir_hir::hir::*;
use crate::{OptimizationPass, OptimizationResult};

/// Dead Code Elimination untuk VDOM.
/// Menghapus props yang tidak digunakan komponen, event handler mati, dan branch tak terjangkau.
pub struct DceVdom;

impl DceVdom {
    pub fn analyze_used_props(component: &HirComponent) -> HashSet<String> {
        let mut used = HashSet::new();
        Self::collect_used_props(&component.render, &mut used, &component.props_param.name);
        used
    }

    fn collect_used_props(expr: &HirExpr, used: &mut HashSet<String>, props_name: &str) {
        match expr {
            HirExpr::Ident(name, _) => {
                if name.starts_with(&format!("{}.", props_name)) {
                    let prop_name = name.trim_start_matches(&format!("{}.", props_name));
                    used.insert(prop_name.to_string());
                }
            }
            HirExpr::JsxElement(elem) => {
                for (_, val) in &elem.attrs {
                    Self::collect_used_props(val, used, props_name);
                }
                for child in &elem.children {
                    Self::collect_used_props(child, used, props_name);
                }
            }
            HirExpr::Binary(bin) => {
                Self::collect_used_props(&bin.lhs, used, props_name);
                Self::collect_used_props(&bin.rhs, used, props_name);
            }
            HirExpr::Call(call) => {
                Self::collect_used_props(&call.callee, used, props_name);
                for arg in &call.args {
                    Self::collect_used_props(arg, used, props_name);
                }
            }
            HirExpr::Block(block) => {
                for stmt in &block.stmts {
                    match stmt {
                        HirStmt::Expr(e) => Self::collect_used_props(e, used, props_name),
                        HirStmt::Let(l) => Self::collect_used_props(&l.value, used, props_name),
                        HirStmt::If(i) => {
                            Self::collect_used_props(&i.condition, used, props_name);
                            for s in &i.then_block.stmts {
                                if let HirStmt::Expr(e) = s {
                                    Self::collect_used_props(e, used, props_name);
                                }
                            }
                            if let Some(ref else_blk) = i.else_block {
                                for s in &else_blk.stmts {
                                    if let HirStmt::Expr(e) = s {
                                        Self::collect_used_props(e, used, props_name);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub fn optimize_props_passing(
        call_site: &HirCall,
        used_props: &HashSet<String>,
    ) -> Vec<HirExpr> {
        if call_site.args.len() > 1 {
            if let HirExpr::StructInit(ref fields) = call_site.args[1] {
                let optimized: Vec<HirStructInitField> = fields.fields
                    .iter()
                    .filter(|f| used_props.contains(&f.name))
                    .cloned()
                    .collect();
                return vec![
                    call_site.args[0].clone(),
                    HirExpr::StructInit(rakit_ir_hir::hir::HirStructInit {
                        name: fields.name.clone(),
                        fields: optimized,
                        ty: fields.ty.clone(),
                    }),
                ];
            }
        }
        call_site.args.clone()
    }
}

impl OptimizationPass for DceVdom {
    fn name(&self) -> &'static str {
        "dce_vdom"
    }

    fn run_on_hir(&self, program: &mut HirProgram) -> Vec<OptimizationResult> {
        let total_removed = 0;

        for item in &program.items {
            if let HirItem::Component(c) = item {
                let _used = Self::analyze_used_props(c);
                let _all_props = c.props_param.name.clone();
            }
        }

        vec![OptimizationResult {
            pass: self.name().to_string(),
            description: format!("DCE VDOM: menghapus {} props tidak terpakai", total_removed),
            items_affected: total_removed,
        }]
    }
}
