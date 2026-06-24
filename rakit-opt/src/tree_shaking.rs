use std::collections::{HashMap, HashSet, VecDeque};
use rakit_ir_hir::hir::*;
use crate::{OptimizationPass, OptimizationResult};

/// Tree shaking — hapus fungsi/komponen/modul yang tidak digunakan.
pub struct TreeShaking;

impl TreeShaking {
    fn is_component_tag(tag: &str) -> bool {
        tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
    }

    fn extract_callees(&self, expr: &HirExpr) -> Vec<String> {
        let mut callees = Vec::new();
        match expr {
            HirExpr::Call(call) => {
                if let HirExpr::Ident(name, _) = call.callee.as_ref() {
                    callees.push(name.clone());
                }
                for arg in &call.args {
                    callees.extend(self.extract_callees(arg));
                }
            }
            HirExpr::JsxElement(elem) => {
                if Self::is_component_tag(&elem.tag) {
                    callees.push(elem.tag.clone());
                }
                for child in &elem.children {
                    callees.extend(self.extract_callees(child));
                }
            }
            HirExpr::Binary(bin) => {
                callees.extend(self.extract_callees(&bin.lhs));
                callees.extend(self.extract_callees(&bin.rhs));
            }
            HirExpr::Block(block) => {
                for stmt in &block.stmts {
                    callees.extend(self.extract_callees_from_stmt(stmt));
                }
            }
            _ => {}
        }
        callees
    }

    fn extract_callees_from_stmt(&self, stmt: &HirStmt) -> Vec<String> {
        match stmt {
            HirStmt::Expr(expr) => self.extract_callees(expr),
            HirStmt::Let(l) => self.extract_callees(&l.value),
            HirStmt::Return(Some(e)) => self.extract_callees(e),
            _ => vec![],
        }
    }

    fn find_entry_points(&self, program: &HirProgram) -> Vec<String> {
        let mut entries = Vec::new();
        for item in &program.items {
            match item {
                HirItem::Function(f) if f.name == "main" => {
                    entries.push(f.name.clone());
                }
                HirItem::Export(_) => {}
                _ => {}
            }
        }
        for item in &program.items {
            if let HirItem::Function(f) = item {
                if f.name == "main" {
                    entries.push(f.name.clone());
                }
            }
        }
        if entries.is_empty() {
            if let Some(HirItem::Function(f)) = program.items.first() {
                entries.push(f.name.clone());
            }
        }
        entries
    }

    fn is_exported(&self, program: &HirProgram, item: &HirItem) -> bool {
        for other in &program.items {
            if let HirItem::Export(e) = other {
                match item {
                    HirItem::Function(f) if e.item_name == f.name => return true,
                    HirItem::Component(c) if e.item_name == c.name => return true,
                    _ => {}
                }
            }
        }
        false
    }
}

impl OptimizationPass for TreeShaking {
    fn name(&self) -> &'static str {
        "tree_shaking"
    }

    fn run_on_hir(&self, program: &mut HirProgram) -> Vec<OptimizationResult> {
        let mut call_graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut all_items: Vec<(String, usize)> = Vec::new();

        for (i, item) in program.items.iter().enumerate() {
            match item {
                HirItem::Function(f) => {
                    let callees = self.extract_callees(&HirExpr::Block(f.body.clone()));
                    call_graph.insert(f.name.clone(), callees);
                    all_items.push((f.name.clone(), i));
                }
                HirItem::Component(c) => {
                    let mut callees = Vec::new();
                    for stmt in &c.body_stmts {
                        callees.extend(self.extract_callees_from_stmt(stmt));
                    }
                    callees.extend(self.extract_callees(&c.render));
                    call_graph.insert(c.name.clone(), callees);
                    all_items.push((c.name.clone(), i));
                }
                _ => {}
            }
        }

        let entry_points = self.find_entry_points(program);
        let mut reachable: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = entry_points.into_iter().collect();

        while let Some(name) = queue.pop_front() {
            if reachable.insert(name.clone()) {
                if let Some(callees) = call_graph.get(&name) {
                    for callee in callees.clone() {
                        queue.push_back(callee);
                    }
                }
            }
        }

        let mut removed = Vec::new();
        let mut i = program.items.len();
        while i > 0 {
            i -= 1;
            let should_remove = match &program.items[i] {
                HirItem::Function(f) => !reachable.contains(&f.name),
                HirItem::Component(c) => !reachable.contains(&c.name),
                _ => false,
            };
            if should_remove && !self.is_exported(program, &program.items[i]) {
                removed.push(i);
            }
        }

        for idx in removed.iter().rev() {
            program.items.remove(*idx);
        }

        vec![OptimizationResult {
            pass: self.name().to_string(),
            description: format!("Menghapus {} item tidak terpakai", removed.len()),
            items_affected: removed.len(),
        }]
    }
}
