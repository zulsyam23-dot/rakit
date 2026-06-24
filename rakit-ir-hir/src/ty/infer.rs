use std::collections::HashMap;
use super::*;
use crate::hir::*;

pub struct InferContext {
    substitutions: HashMap<usize, TypeInfo>,
    next_var: usize,
}

impl InferContext {
    pub fn new() -> Self {
        InferContext {
            substitutions: HashMap::new(),
            next_var: 0,
        }
    }

    pub fn fresh_var(&mut self) -> TypeInfo {
        let id = self.next_var;
        self.next_var += 1;
        TypeInfo::TypeParam(id)
    }

    pub fn unify(&mut self, expected: &TypeInfo, actual: &TypeInfo) -> bool {
        let expected = self.resolve(expected);
        let actual = self.resolve(actual);
        match (&expected, &actual) {
            (a, b) if a == b => true,
            (TypeInfo::TypeParam(id), _) => {
                self.substitutions.insert(*id, actual);
                true
            }
            (_, TypeInfo::TypeParam(id)) => {
                self.substitutions.insert(*id, expected);
                true
            }
            (TypeInfo::Infer, _) | (_, TypeInfo::Infer) => true,
            (TypeInfo::Error, _) | (_, TypeInfo::Error) => true,
            (TypeInfo::Fn(a), TypeInfo::Fn(b)) => {
                a.params.len() == b.params.len()
                    && a.params.iter().zip(&b.params).all(|(x, y)| self.unify(x, y))
                    && self.unify(&a.ret, &b.ret)
            }
            (TypeInfo::Array(a), TypeInfo::Array(b)) => self.unify(a, b),
            (TypeInfo::Optional(a), TypeInfo::Optional(b)) => self.unify(a, b),
            (TypeInfo::Tuple(a), TypeInfo::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| self.unify(x, y))
            }
            (TypeInfo::Struct(a), TypeInfo::Struct(b)) => {
                a.name == b.name
            }
            _ => false,
        }
    }

    pub fn resolve(&self, ty: &TypeInfo) -> TypeInfo {
        match ty {
            TypeInfo::TypeParam(id) => {
                self.substitutions.get(id)
                    .map(|s| self.resolve(s))
                    .unwrap_or(ty.clone())
            }
            TypeInfo::Array(inner) => TypeInfo::Array(Box::new(self.resolve(inner))),
            TypeInfo::Optional(inner) => TypeInfo::Optional(Box::new(self.resolve(inner))),
            TypeInfo::Fn(fn_ty) => TypeInfo::Fn(FnType {
                params: fn_ty.params.iter().map(|p| self.resolve(p)).collect(),
                ret: Box::new(self.resolve(&fn_ty.ret)),
            }),
            TypeInfo::Tuple(tys) => TypeInfo::Tuple(tys.iter().map(|t| self.resolve(t)).collect()),
            _ => ty.clone(),
        }
    }

    pub fn infer_let(&mut self, let_def: &mut HirLet) {
        let value_ty = self.infer_expr_mut(&mut let_def.value);
        if let_def.ty != TypeInfo::Infer {
            if !self.unify(&let_def.ty, &value_ty) {
                let_def.ty = TypeInfo::Error;
            }
        } else {
            let_def.ty = value_ty;
        }
    }

    pub fn infer_expr(&self, expr: &HirExpr) -> TypeInfo {
        match expr {
            HirExpr::Number(_, ty) => self.resolve(ty),
            HirExpr::String(_, ty) => self.resolve(ty),
            HirExpr::Bool(_, ty) => self.resolve(ty),
            HirExpr::Null(ty) => self.resolve(ty),
            HirExpr::Ident(_, ty) => self.resolve(ty),
            HirExpr::Binary(b) => self.resolve(&b.ty),
            HirExpr::Unary(u) => self.resolve(&u.ty),
            HirExpr::Assign(a) => self.resolve(&a.ty),
            HirExpr::Ternary(t) => self.resolve(&t.ty),
            HirExpr::Call(c) => self.resolve(&c.ty),
            HirExpr::Member(m) => self.resolve(&m.ty),
            HirExpr::Index(i) => self.resolve(&i.ty),
            HirExpr::Array(_, ty) => self.resolve(ty),
            HirExpr::StructInit(s) => self.resolve(&s.ty),
            HirExpr::JsxElement(e) => self.resolve(&e.ty),
            HirExpr::HookState(h) => self.resolve(&h.ty),
            HirExpr::HookEffect(_) => TypeInfo::Void,
            HirExpr::HookMemo(h) => self.resolve(&h.ty),
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
            HirExpr::Ident(_, ref mut ty) => { ty.clone() }
            HirExpr::Binary(ref mut b) => {
                let lhs_ty = self.infer_expr_mut(&mut b.lhs);
                let rhs_ty = self.infer_expr_mut(&mut b.rhs);
                if self.unify(&lhs_ty, &rhs_ty) {
                    let r = self.resolve(&lhs_ty);
                    b.ty = r.clone();
                    r
                } else {
                    b.ty = TypeInfo::Error;
                    TypeInfo::Error
                }
            }
            HirExpr::Call(ref mut c) => {
                let callee_ty = self.infer_expr_mut(&mut c.callee);
                for arg in &mut c.args {
                    self.infer_expr_mut(arg);
                }
                match self.resolve(&callee_ty) {
                    TypeInfo::Fn(fn_ty) => {
                        let ret = *fn_ty.ret.clone();
                        c.ty = ret.clone();
                        ret
                    }
                    _ => {
                        c.ty = TypeInfo::Error;
                        TypeInfo::Error
                    }
                }
            }
            _ => TypeInfo::Infer,
        };
        ty
    }
}
