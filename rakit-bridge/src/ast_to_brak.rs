use rakit_ir_hir::hir::*;
use crate::brak_types::*;
use crate::ty_mapping::convert_type;
use crate::error::BridgeError;

pub struct RakitToBrakBridge;

impl RakitToBrakBridge {
    pub fn new() -> Self {
        RakitToBrakBridge
    }

    pub fn convert_program(&self, rakit: &HirProgram) -> Result<BrakProgram, BridgeError> {
        let mut items = Vec::new();
        for item in &rakit.items {
            if let Some(brak_item) = self.convert_item(item)? {
                items.push(brak_item);
            }
        }
        Ok(BrakProgram { items })
    }

    fn convert_item(&self, item: &HirItem) -> std::result::Result<Option<BrakItem>, BridgeError> {
        match item {
            HirItem::Function(f) => {
                let mut params = Vec::new();
                for p in &f.params {
                    params.push(BrakParam {
                        name: p.name.clone(),
                        ty: convert_type(&p.ty),
                    });
                }
                Ok(Some(BrakItem::Function(BrakFnDef {
                    name: f.name.clone(),
                    params,
                    return_ty: Some(convert_type(&f.return_ty)),
                    body: self.convert_block(&f.body)?,
                    is_component: false,
                    hook_calls: vec![],
                })))
            }
            HirItem::Component(c) => {
                let hook_calls: Vec<BrakHookCall> = c.hook_calls.iter()
                    .map(|hc| self.convert_hook_call(hc))
                    .collect::<Result<Vec<_>, _>>()?;
                let body = self.convert_component_body(c)?;
                Ok(Some(BrakItem::Function(BrakFnDef {
                    name: c.name.clone(),
                    params: vec![BrakParam {
                        name: c.props_param.name.clone(),
                        ty: convert_type(&c.props_param.ty),
                    }],
                    return_ty: Some(BrakTy::Named("Node".into())),
                    body,
                    is_component: true,
                    hook_calls,
                })))
            }
            HirItem::Struct(s) => {
                let fields = s.fields.iter().map(|f| BrakStructField {
                    name: f.name.clone(),
                    ty: convert_type(&f.ty),
                }).collect();
                Ok(Some(BrakItem::Struct(BrakStructDef {
                    name: s.name.clone(),
                    fields,
                })))
            }
            HirItem::Enum(e) => {
                let variants = e.variants.iter().map(|v| BrakEnumVariant {
                    name: v.name.clone(),
                    fields: v.fields.iter().map(convert_type).collect(),
                }).collect();
                Ok(Some(BrakItem::Enum(BrakEnumDef {
                    name: e.name.clone(),
                    variants,
                })))
            }
            HirItem::TypeAlias(_) | HirItem::Export(_) | HirItem::Import(_) => {
                Ok(None)
            }
        }
    }

    fn convert_hook_call(&self, hook: &HirHookCall) -> Result<BrakHookCall, BridgeError> {
        match &hook.kind {
            HookKind::State { state_var, setter_var, initial, .. } => {
                Ok(BrakHookCall {
                    kind: BrakHookKind::State {
                        state_var: state_var.clone(),
                        setter_var: setter_var.clone(),
                        initial: Box::new(self.convert_expr(initial)?),
                    },
                })
            }
            HookKind::Effect { callback, deps } => {
                Ok(BrakHookCall {
                    kind: BrakHookKind::Effect {
                        callback: Box::new(self.convert_expr(callback)?),
                        deps: deps.iter().map(|d| self.convert_expr(d)).collect::<Result<Vec<_>, _>>()?,
                    },
                })
            }
            HookKind::Memo { result_var, callback, deps, .. } => {
                Ok(BrakHookCall {
                    kind: BrakHookKind::Memo {
                        result_var: result_var.clone(),
                        callback: Box::new(self.convert_expr(callback)?),
                        deps: deps.iter().map(|d| self.convert_expr(d)).collect::<Result<Vec<_>, _>>()?,
                    },
                })
            }
        }
    }

    fn convert_block(&self, block: &HirBlock) -> Result<BrakBlock, BridgeError> {
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.push(self.convert_stmt(stmt)?);
        }
        Ok(BrakBlock { stmts })
    }

    fn convert_stmt(&self, stmt: &HirStmt) -> Result<BrakStmt, BridgeError> {
        match stmt {
            HirStmt::Let(l) => Ok(BrakStmt::Let(BrakLet {
                name: l.name.clone(),
                mutable: l.mutable,
                ty: if l.ty.is_infer() { None } else { Some(convert_type(&l.ty)) },
                value: self.convert_expr(&l.value)?,
            })),
            HirStmt::Expr(e) => Ok(BrakStmt::Expr(self.convert_expr(e)?)),
            HirStmt::If(i) => Ok(BrakStmt::If(BrakIf {
                condition: self.convert_expr(&i.condition)?,
                then_block: self.convert_block(&i.then_block)?,
                else_block: i.else_block.as_ref().map(|b| self.convert_block(b)).transpose()?,
            })),
            HirStmt::While(w) => Ok(BrakStmt::While(BrakWhile {
                condition: self.convert_expr(&w.condition)?,
                body: self.convert_block(&w.body)?,
            })),
            HirStmt::Match(m) => {
                let arms: Vec<BrakMatchArm> = m.arms.iter()
                    .map(|arm| {
                        Ok(BrakMatchArm {
                            pattern: self.convert_pattern(&arm.pattern),
                            body: self.convert_expr(&arm.body)?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(BrakStmt::Match(BrakMatch {
                    expr: Box::new(self.convert_expr(&m.expr)?),
                    arms,
                }))
            }
            HirStmt::Return(Some(e)) => Ok(BrakStmt::Return(Some(self.convert_expr(e)?))),
            HirStmt::Return(None) => Ok(BrakStmt::Return(None)),
            HirStmt::Block(b) => Ok(BrakStmt::Block(self.convert_block(b)?)),
            HirStmt::Try(t) => Ok(BrakStmt::Try(BrakTry {
                try_block: self.convert_block(&t.try_block)?,
                catch_var: t.catch_var.clone(),
                catch_block: self.convert_block(&t.catch_block)?,
            })),
            HirStmt::Throw(e) => Ok(BrakStmt::Throw(self.convert_expr(e)?)),
            HirStmt::Break => Ok(BrakStmt::Return(None)),
            HirStmt::Continue => Ok(BrakStmt::Return(None)),
        }
    }

    fn convert_pattern(&self, pattern: &HirPattern) -> BrakPattern {
        match pattern {
            HirPattern::Wildcard => BrakPattern::Wildcard,
            HirPattern::Literal(lit) => BrakPattern::Literal(match lit {
                HirLiteral::Number(n) => BrakLiteral::Number(*n),
                HirLiteral::String(s) => BrakLiteral::String(s.clone()),
                HirLiteral::Bool(b) => BrakLiteral::Bool(*b),
                HirLiteral::Null => BrakLiteral::Null,
            }),
            HirPattern::Ident(name) => BrakPattern::Ident(name.clone()),
            HirPattern::Struct { .. } | HirPattern::Enum { .. } => BrakPattern::Ident("_".into()),
        }
    }

    fn convert_expr(&self, expr: &HirExpr) -> Result<BrakExpr, BridgeError> {
        match expr {
            HirExpr::Number(n, _) => Ok(BrakExpr::Number(*n)),
            HirExpr::String(s, _) => Ok(BrakExpr::String(s.clone())),
            HirExpr::Bool(b, _) => Ok(BrakExpr::Bool(*b)),
            HirExpr::Null(_) => Ok(BrakExpr::Null),
            HirExpr::Ident(name, _) => Ok(BrakExpr::Ident(name.clone())),
            HirExpr::Binary(b) => Ok(BrakExpr::Binary(
                self.convert_binop(b.op.clone()),
                Box::new(self.convert_expr(&b.lhs)?),
                Box::new(self.convert_expr(&b.rhs)?),
            )),
            HirExpr::Unary(u) => Ok(BrakExpr::Unary(
                match u.op {
                    HirUnaryOp::Neg => BrakUnaryOp::Neg,
                    HirUnaryOp::Not => BrakUnaryOp::Not,
                },
                Box::new(self.convert_expr(&u.expr)?),
            )),
            HirExpr::Call(c) => {
                let callee = self.convert_expr(&c.callee)?;
                let args: Result<Vec<_>, _> = c.args.iter()
                    .map(|a| self.convert_expr(a))
                    .collect();
                Ok(BrakExpr::Call(Box::new(callee), args?))
            }
            HirExpr::Member(m) => {
                let obj = self.convert_expr(&m.object)?;
                Ok(BrakExpr::Member(Box::new(obj), m.field.clone()))
            }
            HirExpr::Index(i) => {
                let obj = self.convert_expr(&i.object)?;
                let idx = self.convert_expr(&i.index)?;
                Ok(BrakExpr::Index(Box::new(obj), Box::new(idx)))
            }
            HirExpr::Array(items, _) => {
                let brak_items: Result<Vec<_>, _> = items.iter()
                    .map(|i| self.convert_expr(i))
                    .collect();
                Ok(BrakExpr::Array(brak_items?))
            }
            HirExpr::StructInit(s) => {
                let fields: Vec<(String, BrakExpr)> = s.fields.iter()
                    .map(|f| {
                        if f.spread {
                            Ok(("".into(), BrakExpr::Spread(Box::new(self.convert_expr(&f.value)?))))
                        } else {
                            self.convert_expr(&f.value).map(|e| (f.name.clone(), e))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let has_spread = fields.iter().any(|(k, v)| k.is_empty() && matches!(v, BrakExpr::Spread(_)));
                if s.name.is_empty() || has_spread {
                    Ok(BrakExpr::Object(fields))
                } else {
                    Ok(BrakExpr::StructInit(s.name.clone(), fields))
                }
            }
            HirExpr::Ternary(t) => {
                Ok(BrakExpr::Ternary(
                    Box::new(self.convert_expr(&t.condition)?),
                    Box::new(self.convert_expr(&t.then_expr)?),
                    Box::new(self.convert_expr(&t.else_expr)?),
                ))
            }
            HirExpr::Assign(a) => {
                let target = self.convert_expr(&a.target)?;
                let value = self.convert_expr(&a.value)?;
                Ok(BrakExpr::Assign(Box::new(target), Box::new(value)))
            }
            HirExpr::JsxElement(elem) => {
                self.convert_jsx_element(elem)
            }
            HirExpr::Block(b) => Ok(BrakExpr::Block(self.convert_block(b)?)),
            HirExpr::HookState(hs) => {
                Ok(BrakExpr::Ident(hs.state_var.clone()))
            }
            HirExpr::HookEffect(_) => {
                Ok(BrakExpr::Null)
            }
            HirExpr::HookMemo(hm) => {
                Ok(BrakExpr::Ident(hm.result_var.clone()))
            }
        }
    }

    fn convert_jsx_element(&self, elem: &HirJsxElement) -> Result<BrakExpr, BridgeError> {
        let children = BrakExpr::Array(
            elem.children.iter()
                .map(|c| self.convert_expr(c))
                .collect::<Result<Vec<_>, _>>()?
        );
        let attrs = BrakExpr::StructInit("Attrs".into(),
            elem.attrs.iter().map(|(name, val)| {
                self.convert_expr(val).map(|e| (name.clone(), e))
            }).collect::<Result<Vec<_>, _>>()?
        );
        Ok(BrakExpr::Call(
            Box::new(BrakExpr::Ident("h".into())),
            vec![
                BrakExpr::String(elem.tag.clone()),
                attrs,
                children,
            ],
        ))
    }

    fn convert_component_body(&self, c: &HirComponent) -> Result<BrakBlock, BridgeError> {
        let mut stmts = Vec::new();
        for s in &c.body_stmts {
            stmts.push(self.convert_stmt(s)?);
        }
        stmts.push(BrakStmt::Expr(self.convert_expr(&c.render)?));
        Ok(BrakBlock { stmts })
    }

    fn convert_binop(&self, op: HirBinaryOp) -> BrakBinaryOp {
        use HirBinaryOp::*;
        match op {
            Add => BrakBinaryOp::Add, Sub => BrakBinaryOp::Sub,
            Mul => BrakBinaryOp::Mul, Div => BrakBinaryOp::Div,
            Mod => BrakBinaryOp::Mod,
            And => BrakBinaryOp::And, Or => BrakBinaryOp::Or,
            Eq => BrakBinaryOp::Eq, Ne => BrakBinaryOp::Ne,
            Lt => BrakBinaryOp::Lt, Gt => BrakBinaryOp::Gt,
            Le => BrakBinaryOp::Le, Ge => BrakBinaryOp::Ge,
            Concat => BrakBinaryOp::Concat,
            NullCoalescing => BrakBinaryOp::NullCoalescing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rakit_ir_hir::ty::TypeInfo;

    #[test]
    fn test_convert_simple_function() {
        let prog = HirProgram {
            items: vec![
                HirItem::Function(HirFunction {
                    name: "main".into(),
                    params: vec![],
                    return_ty: TypeInfo::I32,
                    body: HirBlock {
                        stmts: vec![
                            HirStmt::Expr(HirExpr::Number(42.0, TypeInfo::I32)),
                        ],
                    },
                    type_params: vec![],
                }),
            ],
        };
        let bridge = RakitToBrakBridge::new();
        let result = bridge.convert_program(&prog);
        assert!(result.is_ok());
        let brak = result.unwrap();
        assert_eq!(brak.items.len(), 1);
    }
}
