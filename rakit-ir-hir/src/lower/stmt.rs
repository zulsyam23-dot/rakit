use rakit_ir_ast as ast;
use crate::hir::*;
use crate::ty::*;
use super::{HirLower, lower_type};

impl HirLower {
    /// Lower a statement, returning multiple HirStmts for destructuring `let`.
    pub fn lower_stmts(&mut self, stmt: &ast::Stmt) -> Vec<HirStmt> {
        match stmt {
            ast::Stmt::Let(let_def) => {
                if let Some(pattern) = &let_def.pattern {
                    let mut out = Vec::new();
                    self.expand_destructuring(
                        pattern, &let_def.value, let_def.mutable, &let_def.ty, &mut out,
                    );
                    return out;
                }
                let value = self.lower_expr(&let_def.value);
                vec![HirStmt::Let(HirLet {
                    name: let_def.name.clone(),
                    mutable: let_def.mutable,
                    ty: let_def.ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Infer),
                    value,
                    span: let_def.span,
                })]
            }
            _ => {
                self.lower_stmt(stmt)
                    .map(|s| vec![s])
                    .unwrap_or_default()
            }
        }
    }

    pub fn lower_stmt(&mut self, stmt: &ast::Stmt) -> Option<HirStmt> {
        let result = match stmt {
            ast::Stmt::Let(let_def) => {
                let value = self.lower_expr(&let_def.value);
                if let Some(_) = &let_def.pattern {
                    // Handled by lower_stmts — return None here
                    return None;
                }
                Some(HirStmt::Let(HirLet {
                    name: let_def.name.clone(),
                    mutable: let_def.mutable,
                    ty: let_def.ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Infer),
                    value,
                    span: let_def.span,
                }))
            }
            ast::Stmt::Expr(expr, _) => {
                Some(HirStmt::Expr(self.lower_expr(expr)))
            }
            ast::Stmt::If(if_stmt) => {
                let condition = self.lower_expr(&if_stmt.condition);
                let then_block = self.lower_block(&if_stmt.then_block);
                let else_block = if_stmt.else_block.as_ref().map(|b| self.lower_block(b));
                Some(HirStmt::If(HirIf {
                    condition,
                    then_block,
                    else_block,
                }))
            }
            ast::Stmt::While(w) => {
                let condition = self.lower_expr(&w.condition);
                let body = self.lower_block(&w.body);
                Some(HirStmt::While(HirWhile { condition, body }))
            }
            ast::Stmt::Match(m) => {
                let expr = self.lower_expr(&m.expr);
                let arms: Vec<HirMatchArm> = m.arms.iter().map(|arm| {
                    HirMatchArm {
                        pattern: lower_pattern(&arm.pattern),
                        body: self.lower_expr(&arm.body),
                    }
                }).collect();
                Some(HirStmt::Match(HirMatch { expr, arms }))
            }
            ast::Stmt::Return(expr, _) => {
                Some(HirStmt::Return(expr.as_ref().map(|e| self.lower_expr(e))))
            }
            ast::Stmt::Block(block) => {
                Some(HirStmt::Block(self.lower_block(block)))
            }
            ast::Stmt::Break(_) => Some(HirStmt::Break),
            ast::Stmt::Continue(_) => Some(HirStmt::Continue),
            ast::Stmt::Try(t) => {
                Some(HirStmt::Try(HirTry {
                    try_block: self.lower_block(&t.try_block),
                    catch_var: t.catch_var.clone(),
                    catch_block: self.lower_block(&t.catch_block),
                }))
            }
            ast::Stmt::Throw(expr, _) => {
                Some(HirStmt::Throw(self.lower_expr(expr)))
            }
            ast::Stmt::HookState(hs) => {
                Some(HirStmt::Let(HirLet {
                    name: hs.state_var.clone(),
                    mutable: true,
                    ty: hs.ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Infer),
                    value: self.lower_expr(&hs.value),
                    span: hs.span,
                }))
            }
            ast::Stmt::FnDef(f) => {
                let fn_ty = TypeInfo::Fn(FnType::new(
                    f.params.iter().map(|p| lower_type(&p.ty)).collect(),
                    f.return_ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Void),
                ));
                Some(HirStmt::Let(HirLet {
                    name: f.name.clone(),
                    mutable: false,
                    ty: fn_ty.clone(),
                    value: HirExpr::Null(fn_ty),
                    span: f.span,
                }))
            }
        };
        result
    }

    pub fn lower_stmt_in_component(
        &mut self,
        stmt: &ast::Stmt,
        hook_calls: &mut Vec<HirHookCall>,
        body_stmts: &mut Vec<HirStmt>,
    ) {
        match stmt {
            ast::Stmt::HookState(hs) => {
                let initial = self.lower_expr(&hs.value);
                let state_var = hs.state_var.clone();
                let setter_var = hs.setter_var.clone();
                let ty = hs.ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Infer);
                hook_calls.push(HirHookCall {
                    kind: HookKind::State {
                        state_var,
                        setter_var,
                        initial: Box::new(initial),
                        ty,
                    },
                    span: hs.span,
                });
            }
            ast::Stmt::Let(let_def) => {
                if let Some(pattern) = &let_def.pattern {
                    self.expand_destructuring(pattern, &let_def.value, let_def.mutable, &let_def.ty, body_stmts);
                } else {
                    let value = self.lower_expr(&let_def.value);
                    body_stmts.push(HirStmt::Let(HirLet {
                        name: let_def.name.clone(),
                        mutable: let_def.mutable,
                        ty: let_def.ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Infer),
                        value,
                        span: let_def.span,
                    }));
                }
            }
            ast::Stmt::Expr(expr, _) => {
                if let Some(hook) = self.try_parse_hook_call(expr) {
                    hook_calls.push(hook);
                } else {
                    body_stmts.push(HirStmt::Expr(self.lower_expr(expr)));
                }
            }
            _ => {
                body_stmts.extend(self.lower_stmts(stmt));
            }
            ast::Stmt::FnDef(f) => {
                let ret_ty = f.return_ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Void);
                let fn_ty = TypeInfo::Fn(FnType::new(
                    f.params.iter().map(|p| lower_type(&p.ty)).collect(),
                    ret_ty,
                ));
                body_stmts.push(HirStmt::Let(HirLet {
                    name: f.name.clone(),
                    mutable: false,
                    ty: fn_ty.clone(),
                    value: HirExpr::Null(fn_ty),
                    span: f.span,
                }));
            }
        }
    }

    fn expand_destructuring(
        &mut self,
        pattern: &ast::Pattern,
        value: &ast::Expr,
        mutable: bool,
        ty: &Option<ast::Type>,
        body_stmts: &mut Vec<HirStmt>,
    ) {
        let lowered_value = self.lower_expr(value);
        match pattern {
            ast::Pattern::Struct { fields, .. } => {
                let lowered_ty = ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Infer);
                for (key, subpat) in fields {
                    let access: HirExpr = if key.parse::<usize>().is_ok() {
                        HirExpr::Index(HirIndex {
                            object: Box::new(lowered_value.clone()),
                            index: Box::new(HirExpr::Number(key.parse().unwrap_or(0.0), TypeInfo::F64)),
                            ty: TypeInfo::Infer,
                        })
                    } else {
                        HirExpr::Member(HirMember {
                            object: Box::new(lowered_value.clone()),
                            field: key.clone(),
                            ty: TypeInfo::Infer,
                        })
                    };
                    match subpat {
                        ast::Pattern::Ident(name) => {
                            body_stmts.push(HirStmt::Let(HirLet {
                                name: name.clone(),
                                mutable,
                                ty: lowered_ty.clone(),
                                value: access,
                                span: Default::default(),
                            }));
                        }
                        ast::Pattern::Struct { fields: inner_fields, .. } => {
                            let inner_value = ast::Expr::Member(Box::new(value.clone()), key.clone());
                            self.expand_destructuring(
                                &ast::Pattern::Struct { name: String::new(), fields: inner_fields.clone() },
                                &inner_value,
                                mutable,
                                ty,
                                body_stmts,
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn try_parse_hook_call(&mut self, expr: &ast::Expr) -> Option<HirHookCall> {
        match expr {
            ast::Expr::Call(callee, args) => {
                match callee.as_ref() {
                    ast::Expr::Ident(name) if name == "keadaan" => {
                        if args.len() == 1 {
                            let initial = self.lower_expr(&args[0]);
                            Some(HirHookCall {
                                kind: HookKind::State {
                                    state_var: format!("__state_{}", self.gen_id()),
                                    setter_var: format!("__set_state_{}", self.gen_id()),
                                    initial: Box::new(initial),
                                    ty: TypeInfo::Infer,
                                },
                                span: Default::default(),
                            })
                        } else {
                            None
                        }
                    }
                    ast::Expr::Ident(name) if name == "efek" => {
                        if args.len() >= 1 {
                            let callback = self.lower_expr(&args[0]);
                            let deps = if args.len() > 1 {
                                args[1..].iter().map(|a| self.lower_expr(a)).collect()
                            } else {
                                Vec::new()
                            };
                            Some(HirHookCall {
                                kind: HookKind::Effect {
                                    callback: Box::new(callback),
                                    deps,
                                },
                                span: Default::default(),
                            })
                        } else {
                            None
                        }
                    }
                    ast::Expr::Ident(name) if name == "ingat" => {
                        if args.len() >= 1 {
                            let callback = self.lower_expr(&args[0]);
                            let deps = if args.len() > 1 {
                                args[1..].iter().map(|a| self.lower_expr(a)).collect()
                            } else {
                                Vec::new()
                            };
                            Some(HirHookCall {
                                kind: HookKind::Memo {
                                    result_var: format!("__memo_{}", self.gen_id()),
                                    callback: Box::new(callback),
                                    deps,
                                    ty: TypeInfo::Infer,
                                },
                                span: Default::default(),
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

fn lower_pattern(pattern: &ast::Pattern) -> HirPattern {
    match pattern {
        ast::Pattern::Wildcard => HirPattern::Wildcard,
        ast::Pattern::Literal(lit) => HirPattern::Literal(match lit {
            ast::Literal::Number(n) => HirLiteral::Number(*n),
            ast::Literal::String(s) => HirLiteral::String(s.clone()),
            ast::Literal::Bool(b) => HirLiteral::Bool(*b),
            ast::Literal::Null => HirLiteral::Null,
            ast::Literal::Char(c) => HirLiteral::String(c.to_string()),
        }),
        ast::Pattern::Ident(name) => HirPattern::Ident(name.clone()),
        ast::Pattern::Struct { name, fields } => {
            HirPattern::Struct {
                name: name.clone(),
                fields: fields.iter().map(|(n, p)| (n.clone(), lower_pattern(p))).collect(),
            }
        }
        ast::Pattern::Enum { name, variant, fields } => {
            HirPattern::Enum {
                name: name.clone(),
                variant: variant.clone(),
                fields: fields.iter().map(lower_pattern).collect(),
            }
        }
    }
}
