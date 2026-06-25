use std::collections::HashMap;
use rakit_core::{Span, Diagnostic};
use crate::hir::*;
use crate::ty::*;

pub struct NameResolver {
    scope_stack: Vec<Scope>,
    pub diagnostics: Vec<Diagnostic>,
    errors: usize,
}

struct Scope {
    bindings: HashMap<String, ScopeEntry>,
    depth: usize,
}

#[allow(dead_code)]
struct ScopeEntry {
    kind: EntryKind,
    ty: Option<TypeInfo>,
    span: Span,
}

#[allow(dead_code)]
enum EntryKind {
    Variable,
    Function,
    Component,
    Struct,
    Enum,
    TypeAlias,
    Imported,
}

impl NameResolver {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        Self::register_builtins(&mut globals);
        NameResolver {
            scope_stack: vec![Scope { bindings: globals, depth: 0 }],
            diagnostics: Vec::new(),
            errors: 0,
        }
    }

    fn register_builtins(scope: &mut HashMap<String, ScopeEntry>) {
        let mut entry = |name: &str, kind: EntryKind| {
            scope.insert(name.to_string(), ScopeEntry {
                kind, ty: None, span: Span::empty(0),
            });
        };
        // Standard JavaScript/browser globals
        for name in &[
            // Types & constructors
            "String", "Array", "Object", "Number", "Boolean", "Date", "Map", "Set",
            "WeakMap", "WeakSet", "Symbol", "BigInt", "RegExp", "Error", "Promise",
            // Math & JSON
            "Math", "JSON", "Atomics", "Reflect", "Proxy",
            // Browser APIs
            "console", "window", "document", "history", "location", "navigator",
            "localStorage", "sessionStorage", "fetch", "WebSocket", "Worker",
            "FileReader", "FormData", "Headers", "Request", "Response", "URL",
            "URLSearchParams", "IntersectionObserver", "MutationObserver",
            "ResizeObserver", "AbortController", "AbortSignal",
            // Timer functions
            "setTimeout", "setInterval", "clearTimeout", "clearInterval",
            "requestAnimationFrame", "cancelAnimationFrame", "requestIdleCallback",
            // Parsing / encoding
            "parseInt", "parseFloat", "isNaN", "isFinite", "decodeURI",
            "encodeURI", "decodeURIComponent", "encodeURIComponent",
            // Rakit-specific runtime
            "Timer", "gunakanFetch", "gunakanKonteks",
            // Rakit stdlib
            "render", "tampilkan", "cetak", "parseJSON", "stringifyJSON",
            "konteks", "tunda", "sekarang", "CSS", "waktu", "Hasil",
            "jalan", "acu", "panggil", "pengedger", "berhenti",
        ] {
            entry(name, EntryKind::Imported);
        }
    }

    pub fn resolve_program(&mut self, program: &mut HirProgram) -> bool {
        for i in 0..program.items.len() {
            match &program.items[i] {
                HirItem::Import(imp) => {
                    for name in &imp.names {
                        self.scope_stack[0].bindings.insert(name.clone(), ScopeEntry {
                            kind: EntryKind::Imported,
                            ty: None,
                            span: Span::empty(0),
                        });
                    }
                }
                _ => Self::register_top_level(&program.items[i], &mut self.scope_stack[0]),
            }
        }
        for item in &mut program.items {
            self.resolve_item(item);
        }
        self.errors == 0
    }

    fn register_top_level(item: &HirItem, scope: &mut Scope) {
        match item {
            HirItem::Function(f) => {
                scope.bindings.insert(f.name.clone(), ScopeEntry {
                    kind: EntryKind::Function,
                    ty: Some(TypeInfo::Fn(FnType::new(
                        f.params.iter().map(|p| p.ty.clone()).collect(),
                        f.return_ty.clone(),
                    ))),
                    span: Span::empty(0),
                });
            }
            HirItem::Component(c) => {
                scope.bindings.insert(c.name.clone(), ScopeEntry {
                    kind: EntryKind::Component,
                    ty: Some(TypeInfo::Fn(FnType::new(
                        vec![c.props_param.ty.clone()],
                        TypeInfo::Node,
                    ))),
                    span: Span::empty(0),
                });
            }
            HirItem::Struct(s) => {
                scope.bindings.insert(s.name.clone(), ScopeEntry {
                    kind: EntryKind::Struct,
                    ty: Some(TypeInfo::Struct(StructType {
                        name: s.name.clone(),
                        fields: s.fields.iter().map(|f| FieldType {
                            name: f.name.clone(),
                            ty: f.ty.clone(),
                        }).collect(),
                        generics: s.generics.iter().map(|g| TypeInfo::Generic(g.clone())).collect(),
                    })),
                    span: Span::empty(0),
                });
            }
            HirItem::Enum(e) => {
                scope.bindings.insert(e.name.clone(), ScopeEntry {
                    kind: EntryKind::Enum,
                    ty: Some(TypeInfo::Enum(EnumType {
                        name: e.name.clone(),
                        variants: e.variants.clone(),
                    })),
                    span: Span::empty(0),
                });
            }
            HirItem::TypeAlias(t) => {
                scope.bindings.insert(t.name.clone(), ScopeEntry {
                    kind: EntryKind::TypeAlias,
                    ty: Some(t.ty.clone()),
                    span: Span::empty(0),
                });
            }
            _ => {}
        }
    }

    fn resolve_item(&mut self, item: &mut HirItem) {
        match item {
            HirItem::Function(f) => {
                self.enter_scope();
                for param in &f.params {
                    self.scope_last().bindings.insert(param.name.clone(), ScopeEntry {
                        kind: EntryKind::Variable,
                        ty: Some(param.ty.clone()),
                        span: param.span,
                    });
                }
                for stmt in &mut f.body.stmts {
                    self.resolve_stmt(stmt);
                }
                self.exit_scope();
            }
            HirItem::Component(c) => {
                self.enter_scope();
                self.scope_last().bindings.insert(c.props_param.name.clone(), ScopeEntry {
                    kind: EntryKind::Variable,
                    ty: Some(c.props_param.ty.clone()),
                    span: c.props_param.span,
                });
                for hc in &c.hook_calls {
                    if let HookKind::State { ref state_var, ref setter_var, ref ty, .. } = hc.kind {
                        self.scope_last().bindings.insert(state_var.clone(), ScopeEntry {
                            kind: EntryKind::Variable, ty: Some(ty.clone()), span: Span::empty(0),
                        });
                        self.scope_last().bindings.insert(setter_var.clone(), ScopeEntry {
                            kind: EntryKind::Variable, ty: Some(TypeInfo::Fn(FnType::new(vec![ty.clone()], TypeInfo::Void))), span: Span::empty(0),
                        });
                    }
                }
                for stmt in &mut c.body_stmts {
                    self.resolve_stmt(stmt);
                }
                self.resolve_expr(&mut c.render);
                self.exit_scope();
            }
            _ => {}
        }
    }

    pub fn resolve_stmt(&mut self, stmt: &mut HirStmt) {
        match stmt {
            HirStmt::Let(l) => {
                self.resolve_expr(&mut l.value);
                let entry_ty = if l.ty != TypeInfo::Infer { Some(l.ty.clone()) } else { None };
                self.scope_last().bindings.insert(l.name.clone(), ScopeEntry {
                    kind: EntryKind::Variable,
                    ty: entry_ty,
                    span: l.span,
                });
            }
            HirStmt::Expr(e) => self.resolve_expr(e),
            HirStmt::If(i) => {
                self.resolve_expr(&mut i.condition);
                self.enter_scope();
                for s in &mut i.then_block.stmts { self.resolve_stmt(s); }
                self.exit_scope();
                if let Some(ref mut else_block) = i.else_block {
                    self.enter_scope();
                    for s in &mut else_block.stmts { self.resolve_stmt(s); }
                    self.exit_scope();
                }
            }
            HirStmt::While(w) => {
                self.resolve_expr(&mut w.condition);
                self.enter_scope();
                for s in &mut w.body.stmts { self.resolve_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Return(Some(e)) => self.resolve_expr(e),
            HirStmt::Return(None) => {}
            HirStmt::Block(b) => {
                self.enter_scope();
                for s in &mut b.stmts { self.resolve_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Match(m) => {
                self.resolve_expr(&mut m.expr);
                for arm in &mut m.arms {
                    self.resolve_pattern(&arm.pattern);
                    self.resolve_expr(&mut arm.body);
                }
            }
            HirStmt::Try(t) => {
                self.enter_scope();
                for s in &mut t.try_block.stmts { self.resolve_stmt(s); }
                self.exit_scope();
                self.enter_scope();
                self.scope_last().bindings.insert(t.catch_var.clone(), ScopeEntry {
                    kind: EntryKind::Variable, ty: None, span: Span::empty(0),
                });
                for s in &mut t.catch_block.stmts { self.resolve_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Break | HirStmt::Continue => {}
            HirStmt::Throw(e) => self.resolve_expr(e),
        }
    }

    pub fn resolve_expr(&mut self, expr: &mut HirExpr) {
        match expr {
            HirExpr::Ident(name, ty) => {
                if let Some(entry) = self.lookup(name) {
                    if let Some(ref entry_ty) = entry.ty {
                        *ty = entry_ty.clone();
                    }
                } else if !is_builtin(name) {
                    self.error(format!("Nama '{}' tidak dikenal dalam scope ini", name));
                    *ty = TypeInfo::Error;
                }
            }
            HirExpr::Binary(b) => {
                self.resolve_expr(&mut b.lhs);
                self.resolve_expr(&mut b.rhs);
            }
            HirExpr::Unary(u) => self.resolve_expr(&mut u.expr),
            HirExpr::Call(c) => {
                self.resolve_expr(&mut c.callee);
                for arg in &mut c.args { self.resolve_expr(arg); }
            }
            HirExpr::Ternary(t) => {
                self.resolve_expr(&mut t.condition);
                self.resolve_expr(&mut t.then_expr);
                self.resolve_expr(&mut t.else_expr);
            }
            HirExpr::Assign(a) => {
                self.resolve_expr(&mut a.target);
                self.resolve_expr(&mut a.value);
            }
            HirExpr::Member(m) => self.resolve_expr(&mut m.object),
            HirExpr::Index(i) => {
                self.resolve_expr(&mut i.object);
                self.resolve_expr(&mut i.index);
            }
            HirExpr::Array(items, _) => {
                for item in items { self.resolve_expr(item); }
            }
            HirExpr::StructInit(s) => {
                for field in &mut s.fields { self.resolve_expr(&mut field.value); }
            }
            HirExpr::JsxElement(e) => {
                for (_, ref mut attr_expr) in &mut e.attrs { self.resolve_expr(attr_expr); }
                for child in &mut e.children { self.resolve_expr(child); }
            }
            HirExpr::Block(b) => {
                self.enter_scope();
                for s in &mut b.stmts { self.resolve_stmt(s); }
                self.exit_scope();
            }
            HirExpr::HookState(h) => self.resolve_expr(&mut *h.initial),
            HirExpr::HookEffect(h) => {
                self.resolve_expr(&mut h.callback);
                for dep in &mut h.deps { self.resolve_expr(dep); }
            }
            HirExpr::HookMemo(h) => {
                self.resolve_expr(&mut h.callback);
                for dep in &mut h.deps { self.resolve_expr(dep); }
            }
            HirExpr::Number(_, _) | HirExpr::String(_, _)
            | HirExpr::Bool(_, _) | HirExpr::Null(_) => {}
        }
    }

    fn resolve_pattern(&self, _pattern: &HirPattern) {}

    fn lookup(&self, name: &str) -> Option<&ScopeEntry> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(entry) = scope.bindings.get(name) {
                return Some(entry);
            }
        }
        None
    }

    fn enter_scope(&mut self) {
        let depth = self.scope_stack.last().map(|s| s.depth + 1).unwrap_or(0);
        self.scope_stack.push(Scope { bindings: HashMap::new(), depth });
    }

    fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn scope_last(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().unwrap()
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.errors += 1;
        self.diagnostics.push(Diagnostic::error(msg));
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "h" | "cetak" | "benar" | "salah" | "batal"
        | "keadaan" | "efek" | "ingat" | "render"
        | "tampilkan" | "konteks" | "parseJSON" | "stringifyJSON"
        | "tunda" | "sekarang" | "CSS" | "waktu" | "Hasil"
        | "jalan" | "acu" | "panggil" | "pengedger" | "berhenti")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_simple() {
        let mut program = HirProgram { items: vec![
            HirItem::Function(HirFunction {
                name: "main".into(),
                params: vec![],
                return_ty: TypeInfo::I32,
                body: HirBlock { stmts: vec![
                    HirStmt::Let(HirLet {
                        name: "x".into(), mutable: false,
                        ty: TypeInfo::I32,
                        value: HirExpr::Number(42.0, TypeInfo::I32),
                        span: Span::empty(0),
                    }),
                    HirStmt::Expr(HirExpr::Ident("x".into(), TypeInfo::Infer)),
                ] },
                type_params: vec![],
            }),
        ] };
        let mut resolver = NameResolver::new();
        let ok = resolver.resolve_program(&mut program);
        assert!(ok);
    }

    #[test]
    fn test_resolve_unknown() {
        let mut program = HirProgram { items: vec![
            HirItem::Function(HirFunction {
                name: "main".into(),
                params: vec![],
                return_ty: TypeInfo::I32,
                body: HirBlock { stmts: vec![
                    HirStmt::Expr(HirExpr::Ident("unknown_var".into(), TypeInfo::Infer)),
                ] },
                type_params: vec![],
            }),
        ] };
        let mut resolver = NameResolver::new();
        let ok = resolver.resolve_program(&mut program);
        assert!(!ok);
    }
}
