use std::collections::HashMap;
use rakit_core::Diagnostic;
use crate::hir::*;
use super::*;

pub struct TypeChecker {
    scope: Vec<Scope>,
    struct_defs: HashMap<String, StructType>,
    enum_defs: HashMap<String, EnumType>,
    fn_sigs: HashMap<String, FnType>,
    pub diagnostics: Vec<Diagnostic>,
    errors: usize,
}

struct Scope {
    bindings: HashMap<String, TypeInfo>,
    depth: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scope: vec![Scope { bindings: HashMap::new(), depth: 0 }],
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            fn_sigs: HashMap::new(),
            diagnostics: Vec::new(),
            errors: 0,
        }
    }

    pub fn register_top_level(&mut self, item: &HirItem) {
        match item {
            HirItem::Function(f) => {
                let fn_ty = FnType::new(
                    f.params.iter().map(|p| p.ty.clone()).collect(),
                    f.return_ty.clone(),
                );
                self.fn_sigs.insert(f.name.clone(), fn_ty.clone());
                self.scope[0].bindings.insert(f.name.clone(), TypeInfo::Fn(fn_ty));
            }
            HirItem::Struct(s) => {
                let st = StructType {
                    name: s.name.clone(),
                    fields: s.fields.iter().map(|f| FieldType {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                    }).collect(),
                    generics: s.generics.iter().map(|g| TypeInfo::Generic(g.clone())).collect(),
                };
                self.struct_defs.insert(s.name.clone(), st.clone());
                self.scope[0].bindings.insert(s.name.clone(), TypeInfo::Struct(st));
            }
            HirItem::Enum(e) => {
                let et = EnumType {
                    name: e.name.clone(),
                    variants: e.variants.clone(),
                };
                self.enum_defs.insert(e.name.clone(), et.clone());
                self.scope[0].bindings.insert(e.name.clone(), TypeInfo::Enum(et));
            }
            _ => {}
        }
    }

    pub fn check_program(&mut self, program: &mut HirProgram) -> bool {
        let items_copy = program.items.clone();
        for item in &items_copy {
            self.register_top_level(item);
        }
        for item in &mut program.items {
            self.check_item(item);
        }
        self.errors == 0
    }

    pub fn check_item(&mut self, item: &mut HirItem) {
        match item {
            HirItem::Function(f) => {
                self.enter_scope();
                for param in &f.params {
                    self.scope_last().bindings.insert(param.name.clone(), param.ty.clone());
                }
                for stmt in &mut f.body.stmts {
                    self.check_stmt(stmt);
                }
                self.exit_scope();
            }
            HirItem::Component(c) => {
                self.enter_scope();
                self.scope_last().bindings.insert(c.props_param.name.clone(), c.props_param.ty.clone());
                for hc in &c.hook_calls {
                    if let HookKind::State { ref state_var, ref setter_var, ref ty, .. } = hc.kind {
                        self.scope_last().bindings.insert(state_var.clone(), ty.clone());
                        self.scope_last().bindings.insert(
                            setter_var.clone(),
                            TypeInfo::Fn(FnType::new(vec![ty.clone()], TypeInfo::Void)),
                        );
                    }
                }
                for stmt in &mut c.body_stmts {
                    self.check_stmt(stmt);
                }
                self.exit_scope();
            }
            _ => {}
        }
    }

    pub fn check_stmt(&mut self, stmt: &mut HirStmt) {
        match stmt {
            HirStmt::Let(l) => {
                let val_ty = self.infer_expr_mut(&mut l.value);
                if l.ty != TypeInfo::Infer && !self.unify(&l.ty, &val_ty) {
                    self.error(format!(
                        "Tipe tidak cocok: diharapkan {:?}, ditemukan {:?}",
                        l.ty, val_ty
                    ));
                }
                self.scope_last().bindings.insert(l.name.clone(), if l.ty != TypeInfo::Infer { l.ty.clone() } else { val_ty });
            }
            HirStmt::Expr(e) => { self.infer_expr_mut(e); }
            HirStmt::If(i) => {
                let cond_ty = self.infer_expr_mut(&mut i.condition);
                if cond_ty != TypeInfo::Bool && !cond_ty.is_error() {
                    self.error("Kondisi 'jika' harus bertipe Bool");
                }
                self.enter_scope();
                for s in &mut i.then_block.stmts { self.check_stmt(s); }
                self.exit_scope();
                if let Some(ref mut else_block) = i.else_block {
                    self.enter_scope();
                    for s in &mut else_block.stmts { self.check_stmt(s); }
                    self.exit_scope();
                }
            }
            HirStmt::While(w) => {
                let cond_ty = self.infer_expr_mut(&mut w.condition);
                if cond_ty != TypeInfo::Bool && !cond_ty.is_error() {
                    self.error("Kondisi 'ulang' harus bertipe Bool");
                }
                self.enter_scope();
                for s in &mut w.body.stmts { self.check_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Return(Some(e)) => { self.infer_expr_mut(e); }
            HirStmt::Return(None) => {}
            HirStmt::Block(b) => {
                self.enter_scope();
                for s in &mut b.stmts { self.check_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Match(m) => {
                self.infer_expr_mut(&mut m.expr);
                for arm in &mut m.arms {
                    self.check_pattern(&arm.pattern);
                    self.infer_expr_mut(&mut arm.body);
                }
            }
            HirStmt::Break | HirStmt::Continue => {}
            HirStmt::Try(t) => {
                self.enter_scope();
                for s in &mut t.try_block.stmts { self.check_stmt(s); }
                self.exit_scope();
                self.enter_scope();
                self.scope_last().bindings.insert(t.catch_var.clone(), TypeInfo::Infer);
                for s in &mut t.catch_block.stmts { self.check_stmt(s); }
                self.exit_scope();
            }
            HirStmt::Throw(e) => { self.infer_expr_mut(e); }
        }
    }

    pub fn infer_expr(&self, expr: &HirExpr) -> TypeInfo {
        match expr {
            HirExpr::Number(_, ty) => ty.clone(),
            HirExpr::String(_, ty) => ty.clone(),
            HirExpr::Bool(_, ty) => ty.clone(),
            HirExpr::Null(ty) => ty.clone(),
            HirExpr::Ident(name, ty) => {
                if ty.is_infer() {
                    self.lookup(name)
                } else {
                    ty.clone()
                }
            }
            HirExpr::Binary(b) => b.ty.clone(),
            HirExpr::Unary(u) => u.ty.clone(),
            HirExpr::Assign(a) => a.ty.clone(),
            HirExpr::Ternary(t) => t.ty.clone(),
            HirExpr::Call(c) => c.ty.clone(),
            HirExpr::Member(m) => m.ty.clone(),
            HirExpr::Index(i) => i.ty.clone(),
            HirExpr::Array(_, ty) => ty.clone(),
            HirExpr::StructInit(s) => s.ty.clone(),
            HirExpr::JsxElement(e) => e.ty.clone(),
            HirExpr::HookState(h) => h.ty.clone(),
            HirExpr::HookEffect(_) => TypeInfo::Void,
            HirExpr::HookMemo(h) => h.ty.clone(),
            HirExpr::Block(b) => {
                b.stmts.last().map(|s| match s {
                    HirStmt::Expr(e) => self.infer_expr(e),
                    _ => TypeInfo::Void,
                }).unwrap_or(TypeInfo::Void)
            }
        }
    }

    pub fn infer_expr_mut(&mut self, expr: &mut HirExpr) -> TypeInfo {
        let ty = match expr {
            HirExpr::Number(_, ref mut ty) => { *ty = TypeInfo::F64; TypeInfo::F64 }
            HirExpr::String(_, ref mut ty) => { *ty = TypeInfo::String; TypeInfo::String }
            HirExpr::Bool(_, ref mut ty) => { *ty = TypeInfo::Bool; TypeInfo::Bool }
            HirExpr::Null(ref mut ty) => { *ty = TypeInfo::Optional(Box::new(TypeInfo::Infer)); ty.clone() }
            HirExpr::Ident(name, ref mut ty) => {
                let resolved = self.lookup(name);
                *ty = resolved.clone();
                resolved
            }
            HirExpr::Binary(ref mut b) => {
                let lhs_ty = self.infer_expr_mut(&mut b.lhs);
                let rhs_ty = self.infer_expr_mut(&mut b.rhs);
                let result = self.check_binary_op(b.op.clone(), &lhs_ty, &rhs_ty);
                b.ty = result.clone();
                result
            }
            HirExpr::Unary(ref mut u) => {
                let inner = self.infer_expr_mut(&mut u.expr);
                let result = match u.op {
                    HirUnaryOp::Neg if inner.is_numeric() => inner,
                    HirUnaryOp::Not if inner == TypeInfo::Bool => TypeInfo::Bool,
                    _ => { self.error("Operator tidak cocok dengan tipe"); TypeInfo::Error }
                };
                u.ty = result.clone();
                result
            }
            HirExpr::Call(ref mut c) => {
                let callee_ty = self.infer_expr_mut(&mut c.callee);
                let mut arg_tys = Vec::new();
                for arg in &mut c.args {
                    arg_tys.push(self.infer_expr_mut(arg));
                }
                let result = self.check_call(&callee_ty, &arg_tys);
                c.ty = result.clone();
                result
            }
            HirExpr::Ternary(ref mut t) => {
                let cond_ty = self.infer_expr_mut(&mut t.condition);
                if cond_ty != TypeInfo::Bool && !cond_ty.is_error() {
                    self.error("Kondisi ternary harus bertipe Bool");
                }
                let then_ty = self.infer_expr_mut(&mut t.then_expr);
                let else_ty = self.infer_expr_mut(&mut t.else_expr);
                let result = if self.unify(&then_ty, &else_ty) { then_ty } else { else_ty };
                t.ty = result.clone();
                result
            }
            HirExpr::Assign(ref mut a) => {
                let _target_ty = self.infer_expr_mut(&mut a.target);
                let val_ty = self.infer_expr_mut(&mut a.value);
                a.ty = TypeInfo::Void;
                val_ty
            }
            HirExpr::Member(ref mut m) => {
                let obj_ty = self.infer_expr_mut(&mut m.object);
                let result = self.check_member(&obj_ty, &m.field);
                m.ty = result.clone();
                result
            }
            HirExpr::Index(ref mut i) => {
                let obj_ty = self.infer_expr_mut(&mut i.object);
                let _idx_ty = self.infer_expr_mut(&mut i.index);
                let result = match obj_ty {
                    TypeInfo::Array(ref inner) => *inner.clone(),
                    _ => { self.error("Hanya array yang bisa di-index"); TypeInfo::Error }
                };
                i.ty = result.clone();
                result
            }
            HirExpr::Array(ref mut items, ref mut ty) => {
                if items.is_empty() {
                    *ty = TypeInfo::Array(Box::new(TypeInfo::Infer));
                    ty.clone()
                } else {
                    let item_ty = self.infer_expr_mut(&mut items[0]);
                    for item in &mut items[1..] {
                        self.infer_expr_mut(item);
                    }
                    *ty = TypeInfo::Array(Box::new(item_ty.clone()));
                    ty.clone()
                }
            }
            HirExpr::StructInit(ref mut s) => {
                if s.name == "Attrs" {
                    for field in &mut s.fields {
                        self.infer_expr_mut(&mut field.value);
                    }
                    s.ty = TypeInfo::Struct(StructType {
                        name: "Attrs".into(),
                        fields: s.fields.iter().map(|f| FieldType {
                            name: f.name.clone(),
                            ty: TypeInfo::Infer,
                        }).collect(),
                        generics: Vec::new(),
                    });
                    return s.ty.clone();
                }
                let st_opt = self.struct_defs.get(&s.name).cloned();
                if let Some(st) = st_opt {
                    for field in &mut s.fields {
                        self.infer_expr_mut(&mut field.value);
                    }
                    s.ty = TypeInfo::Struct(st);
                    s.ty.clone()
                } else {
                    self.error(format!("Struktur '{}' tidak dikenal", s.name));
                    TypeInfo::Error
                }
            }
            HirExpr::JsxElement(ref mut e) => {
                for (_, ref mut attr_expr) in &mut e.attrs {
                    self.infer_expr_mut(attr_expr);
                }
                for child in &mut e.children {
                    self.infer_expr_mut(child);
                }
                e.ty = TypeInfo::Node;
                TypeInfo::Node
            }
            HirExpr::HookState(ref mut h) => {
                let init_ty = self.infer_expr_mut(&mut *h.initial);
                h.ty = init_ty.clone();
                init_ty
            }
            HirExpr::HookEffect(ref mut h) => {
                self.infer_expr_mut(&mut h.callback);
                for dep in &mut h.deps {
                    self.infer_expr_mut(dep);
                }
                TypeInfo::Void
            }
            HirExpr::HookMemo(ref mut h) => {
                let result_ty = self.infer_expr_mut(&mut h.callback);
                h.ty = result_ty.clone();
                for dep in &mut h.deps {
                    self.infer_expr_mut(dep);
                }
                result_ty
            }
            HirExpr::Block(ref mut b) => {
                self.enter_scope();
                for s in &mut b.stmts {
                    self.check_stmt(s);
                }
                let result = b.stmts.last().map(|s| match s {
                    HirStmt::Expr(e) => self.infer_expr(e),
                    _ => TypeInfo::Void,
                }).unwrap_or(TypeInfo::Void);
                self.exit_scope();
                result
            }
        };
        ty
    }

    fn check_binary_op(&self, op: HirBinaryOp, lhs: &TypeInfo, rhs: &TypeInfo) -> TypeInfo {
        match op {
            HirBinaryOp::Add | HirBinaryOp::Sub | HirBinaryOp::Mul
            | HirBinaryOp::Div | HirBinaryOp::Mod => {
                if !lhs.is_numeric() || !rhs.is_numeric() {
                    TypeInfo::Error
                } else {
                    type_widen(lhs, rhs)
                }
            }
            HirBinaryOp::And | HirBinaryOp::Or => {
                if *lhs == TypeInfo::Bool && *rhs == TypeInfo::Bool {
                    TypeInfo::Bool
                } else {
                    TypeInfo::Error
                }
            }
            HirBinaryOp::Eq | HirBinaryOp::Ne
            | HirBinaryOp::Lt | HirBinaryOp::Gt
            | HirBinaryOp::Le | HirBinaryOp::Ge => {
                TypeInfo::Bool
            }
            HirBinaryOp::Concat => TypeInfo::String,
        }
    }

    fn check_call(&self, callee_ty: &TypeInfo, _arg_tys: &[TypeInfo]) -> TypeInfo {
        match callee_ty {
            TypeInfo::Fn(fn_ty) => {
                *fn_ty.ret.clone()
            }
            _ => TypeInfo::Error,
        }
    }

    fn check_member(&self, obj_ty: &TypeInfo, field: &str) -> TypeInfo {
        match obj_ty {
            TypeInfo::Struct(st) => {
                st.fields.iter().find(|f| f.name == field)
                    .map(|f| f.ty.clone())
                    .unwrap_or(TypeInfo::Error)
            }
            _ => TypeInfo::Error,
        }
    }

    fn check_pattern(&self, pattern: &HirPattern) {
        match pattern {
            HirPattern::Wildcard => {}
            HirPattern::Literal(_) => {}
            HirPattern::Ident(_) => {}
            HirPattern::Struct { name, fields } => {
                if let Some(st) = self.struct_defs.get(name) {
                    for (fname, _) in fields {
                        if !st.fields.iter().any(|f| f.name == *fname) {
                        }
                    }
                }
            }
            HirPattern::Enum { name, variant, fields: _ } => {
                if let Some(et) = self.enum_defs.get(name) {
                    if !et.variants.iter().any(|v| v.name == *variant) {
                    }
                }
            }
        }
    }

    fn lookup(&self, name: &str) -> TypeInfo {
        for scope in self.scope.iter().rev() {
            if let Some(ty) = scope.bindings.get(name) {
                return ty.clone();
            }
        }
        match name {
            "h" => TypeInfo::Fn(FnType::new(
                vec![TypeInfo::String, TypeInfo::Struct(StructType {
                    name: "Attrs".into(), fields: vec![], generics: vec![],
                }), TypeInfo::Array(Box::new(TypeInfo::Node))],
                TypeInfo::Node,
            )),
            "cetak" => TypeInfo::Fn(FnType::new(
                vec![TypeInfo::String], TypeInfo::Void,
            )),
            "benar" => TypeInfo::Bool,
            "salah" => TypeInfo::Bool,
            "batal" => TypeInfo::Optional(Box::new(TypeInfo::Infer)),
            _ => TypeInfo::Error,
        }
    }

    fn unify(&self, expected: &TypeInfo, actual: &TypeInfo) -> bool {
        match (expected, actual) {
            (a, b) if a == b => true,
            (TypeInfo::Infer, _) | (_, TypeInfo::Infer) => true,
            (TypeInfo::I32, TypeInfo::I64) => true,
            (TypeInfo::I64, TypeInfo::I32) => true,
            (TypeInfo::F32, TypeInfo::F64) => true,
            (TypeInfo::F64, TypeInfo::F32) => true,
            (TypeInfo::Optional(a), _b) if **a == TypeInfo::Infer => true,
            (TypeInfo::Optional(a), b) if a.as_ref() == b => true,
            (a, TypeInfo::Optional(b)) if a == b.as_ref() => true,
            (TypeInfo::Error, _) | (_, TypeInfo::Error) => true,
            _ => false,
        }
    }

    fn enter_scope(&mut self) {
        let depth = self.scope.last().map(|s| s.depth + 1).unwrap_or(0);
        self.scope.push(Scope { bindings: HashMap::new(), depth });
    }

    fn exit_scope(&mut self) {
        self.scope.pop();
    }

    fn scope_last(&mut self) -> &mut Scope {
        self.scope.last_mut().unwrap()
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.errors += 1;
        self.diagnostics.push(Diagnostic::error(msg));
    }
}
