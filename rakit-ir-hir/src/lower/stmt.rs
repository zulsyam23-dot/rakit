use rakit_ir_ast as ast;
use crate::hir::*;
use crate::ty::*;
use super::{HirLower, lower_type};

impl HirLower {
    pub fn lower_stmt(&mut self, stmt: &ast::Stmt) -> Option<HirStmt> {
        let result = match stmt {
            ast::Stmt::Let(let_def) => {
                let value = self.lower_expr(&let_def.value);
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
            ast::Stmt::Let(let_def) => {
                let value = self.lower_expr(&let_def.value);
                body_stmts.push(HirStmt::Let(HirLet {
                    name: let_def.name.clone(),
                    mutable: let_def.mutable,
                    ty: let_def.ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Infer),
                    value,
                    span: let_def.span,
                }));
            }
            ast::Stmt::Expr(expr, _) => {
                if let Some(hook) = self.try_parse_hook_call(expr) {
                    hook_calls.push(hook);
                } else {
                    body_stmts.push(HirStmt::Expr(self.lower_expr(expr)));
                }
            }
            _ => {
                if let Some(s) = self.lower_stmt(stmt) {
                    body_stmts.push(s);
                }
            }
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
