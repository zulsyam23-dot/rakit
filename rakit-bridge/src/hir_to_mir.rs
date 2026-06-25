use rakit_ir_hir::hir::*;
use rakit_ir_hir::ty::TypeInfo;
use brak_ir_mir::mir::*;
use brak_core::Span as BrakSpan;
use rakit_core::Span as RakitSpan;
use crate::error::BridgeError;

/// Convert rakit Span to brak Span
fn convert_span(s: RakitSpan) -> BrakSpan {
    BrakSpan::new(
        brak_core::SourceLoc::new(0, 0, s.start),
        brak_core::SourceLoc::new(0, 0, s.end),
    )
}

/// Get a dummy brak Span
fn dummy_span() -> BrakSpan {
    brak_core::DUMMY_SPAN
}

pub struct RakitToBrakBridge {
    next_local: usize,
}

impl RakitToBrakBridge {
    pub fn new() -> Self {
        RakitToBrakBridge { next_local: 0 }
    }

    fn alloc_local(&mut self) -> usize {
        let id = self.next_local;
        self.next_local += 1;
        id
    }

    pub fn convert_program(&mut self, rakit: &HirProgram) -> Result<MirProgram, BridgeError> {
        let mut functions = Vec::new();

        for item in &rakit.items {
            match item {
                HirItem::Function(f) => {
                    functions.push(self.convert_function(f)?);
                }
                HirItem::Component(c) => {
                    functions.push(self.convert_component(c)?);
                }
                _ => {}
            }
        }

        Ok(MirProgram {
            functions,
            extern_functions: vec![],
            structs: vec![],
            enums: vec![],
        })
    }

    fn convert_function(&mut self, f: &HirFunction) -> Result<MirFunction, BridgeError> {
        self.next_local = 0;
        let mut params = Vec::new();
        let mut locals = Vec::new();

        for p in &f.params {
            let local_id = self.alloc_local();
            params.push(local_id);
            locals.push(MirLocal {
                name: p.name.clone(),
                ty: convert_type(&p.ty),
            });
        }

        let entry_block_id = self.alloc_local();
        let mut blocks = self.convert_block(&f.body, entry_block_id)?;

        // Ensure we have at least one block with a terminator
        if blocks.is_empty() {
            blocks.push(MirBlock {
                id: entry_block_id,
                name: format!("block{}", entry_block_id),
                insts: vec![],
                terminator: MirTerminator::Return { value: None, span: dummy_span() },
                span: dummy_span(),
            });
        }

        Ok(MirFunction {
            name: f.name.clone(),
            params,
            ret_ty: convert_type(&f.return_ty),
            blocks,
            locals,
            span: dummy_span(),
        })
    }

    fn convert_component(&mut self, c: &HirComponent) -> Result<MirFunction, BridgeError> {
        self.next_local = 0;
        let mut params = Vec::new();
        let mut locals = Vec::new();

        // Props param
        let props_id = self.alloc_local();
        params.push(props_id);
        locals.push(MirLocal {
            name: c.props_param.name.clone(),
            ty: convert_type(&c.props_param.ty),
        });

        // State hook locals
        for hc in &c.hook_calls {
            match &hc.kind {
                HookKind::State { state_var, setter_var, .. } => {
                    locals.push(MirLocal { name: state_var.clone(), ty: MirType::I64 });
                    locals.push(MirLocal { name: setter_var.clone(), ty: MirType::I64 });
                }
                HookKind::Memo { result_var, .. } => {
                    locals.push(MirLocal { name: result_var.clone(), ty: MirType::I64 });
                }
                _ => {}
            }
        }

        // Convert body statements
        let mut blocks = Vec::new();
        let mut current_insts = Vec::new();
        let mut block_id = self.alloc_local();

        // Convert hook state assignments
        for hc in &c.hook_calls {
            if let HookKind::State { state_var, initial, .. } = &hc.kind {
                let local = self.find_or_alloc_local(&state_var, &mut locals);
                let (value, insts) = self.lower_expr(initial);
                current_insts.extend(insts);
                current_insts.push(MirInst::Assign {
                    dest: local,
                    value,
                    span: dummy_span(),
                });
            }
        }

        // Convert body statements
        for stmt in &c.body_stmts {
            self.convert_stmt(stmt, &mut blocks, &mut current_insts, &mut block_id, &mut locals);
        }

        // Convert render expression as final return
        let (render_val, render_insts) = self.lower_expr(&c.render);
        current_insts.extend(render_insts);
        let render_local = self.alloc_local();
        current_insts.push(MirInst::Assign {
            dest: render_local,
            value: render_val,
            span: dummy_span(),
        });

        blocks.push(MirBlock {
            id: block_id,
            name: format!("block{}", block_id),
            insts: current_insts,
            terminator: MirTerminator::Return {
                value: Some(render_local),
                span: dummy_span(),
            },
            span: dummy_span(),
        });

        Ok(MirFunction {
            name: c.name.clone(),
            params,
            ret_ty: MirType::Named("Node".into()),
            blocks,
            locals,
            span: dummy_span(),
        })
    }

    fn convert_block(&mut self, block: &HirBlock, mut block_id: usize) -> Result<Vec<MirBlock>, BridgeError> {
        let mut blocks = Vec::new();
        let mut current_insts = Vec::new();

        for stmt in &block.stmts {
            self.convert_stmt(stmt, &mut blocks, &mut current_insts, &mut block_id, &mut vec![]);
        }

        // Add final block with return
        blocks.push(MirBlock {
            id: block_id,
            name: format!("block{}", block_id),
            insts: current_insts,
            terminator: MirTerminator::Return { value: None, span: dummy_span() },
            span: dummy_span(),
        });

        Ok(blocks)
    }

    fn convert_stmt(
        &mut self,
        stmt: &HirStmt,
        blocks: &mut Vec<MirBlock>,
        current_insts: &mut Vec<MirInst>,
        block_id: &mut usize,
        _locals: &mut Vec<MirLocal>,
    ) {
        match stmt {
            HirStmt::Let(l) => {
                let local = self.alloc_local();
                let (value, insts) = self.lower_expr(&l.value);
                current_insts.extend(insts);
                current_insts.push(MirInst::Assign {
                    dest: local,
                    value,
                    span: dummy_span(),
                });
            }
            HirStmt::Expr(e) => {
                let (value, insts) = self.lower_expr(e);
                current_insts.extend(insts);
                let local = self.alloc_local();
                current_insts.push(MirInst::Assign {
                    dest: local,
                    value,
                    span: dummy_span(),
                });
            }
            HirStmt::Return(e) => {
                let value = if let Some(expr) = e {
                    let (val, insts) = self.lower_expr(expr);
                    current_insts.extend(insts);
                    let local = self.alloc_local();
                    current_insts.push(MirInst::Assign {
                        dest: local,
                        value: val,
                        span: dummy_span(),
                    });
                    Some(local)
                } else {
                    None
                };

                let current_bid = *block_id;
                blocks.push(MirBlock {
                    id: current_bid,
                    name: format!("block{}", current_bid),
                    insts: std::mem::take(current_insts),
                    terminator: MirTerminator::Return { value, span: dummy_span() },
                    span: dummy_span(),
                });
                *block_id = self.alloc_local();
            }
            HirStmt::If(i) => {
                let (cond_val, cond_insts) = self.lower_expr(&i.condition);
                current_insts.extend(cond_insts);

                let cond_local = self.alloc_local();
                current_insts.push(MirInst::Assign {
                    dest: cond_local,
                    value: cond_val,
                    span: dummy_span(),
                });

                let mut then_block_id = self.alloc_local();
                let mut else_block_id = self.alloc_local();
                let merge_block_id = self.alloc_local();

                // Current block branches
                let block_id_val = *block_id;
                blocks.push(MirBlock {
                    id: block_id_val,
                    name: format!("block{}", block_id_val),
                    insts: std::mem::take(current_insts),
                    terminator: MirTerminator::Branch {
                        cond: cond_local,
                        then: then_block_id,
                        else_: else_block_id,
                        span: dummy_span(),
                    },
                    span: dummy_span(),
                });

                // Then block
                let mut then_insts = Vec::new();
                let mut then_blocks = Vec::new();
                for s in &i.then_block.stmts {
                    self.convert_stmt(s, &mut then_blocks, &mut then_insts, &mut then_block_id, &mut vec![]);
                }
                then_blocks.push(MirBlock {
                    id: then_block_id,
                    name: format!("block{}", then_block_id),
                    insts: then_insts,
                    terminator: MirTerminator::Jump { target: merge_block_id, span: dummy_span() },
                    span: dummy_span(),
                });
                blocks.append(&mut then_blocks);

                // Else block
                let mut else_insts = Vec::new();
                let mut else_blocks = Vec::new();
                if let Some(else_block) = &i.else_block {
                    for s in &else_block.stmts {
                        self.convert_stmt(s, &mut else_blocks, &mut else_insts, &mut else_block_id, &mut vec![]);
                    }
                }
                else_blocks.push(MirBlock {
                    id: else_block_id,
                    name: format!("block{}", else_block_id),
                    insts: else_insts,
                    terminator: MirTerminator::Jump { target: merge_block_id, span: dummy_span() },
                    span: dummy_span(),
                });
                blocks.append(&mut else_blocks);

                *block_id = merge_block_id;
            }
            _ => {}
        }
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> (MirValue, Vec<MirInst>) {
        match expr {
            HirExpr::Number(n, _) => {
                if *n == (*n as i64) as f64 {
                    (MirValue::Int(*n as i64), vec![])
                } else {
                    (MirValue::Float(*n), vec![])
                }
            }
            HirExpr::String(s, _) => (MirValue::String(s.clone()), vec![]),
            HirExpr::Bool(b, _) => (MirValue::Bool(*b), vec![]),
            HirExpr::Null(_) => (MirValue::Int(0), vec![]),
            HirExpr::Ident(name, _) => {
                let local = self.alloc_local();
                (MirValue::Local(local), vec![MirInst::Assign {
                    dest: local,
                    value: MirValue::Local(name.len()), // Placeholder
                    span: dummy_span(),
                }])
            }
            HirExpr::Binary(b) => {
                let (lhs_val, lhs_insts) = self.lower_expr(&b.lhs);
                let (rhs_val, rhs_insts) = self.lower_expr(&b.rhs);

                let lhs_local = self.alloc_local();
                let rhs_local = self.alloc_local();
                let result_local = self.alloc_local();

                let mut all_insts = lhs_insts;
                all_insts.push(MirInst::Assign { dest: lhs_local, value: lhs_val, span: dummy_span() });
                all_insts.extend(rhs_insts);
                all_insts.push(MirInst::Assign { dest: rhs_local, value: rhs_val, span: dummy_span() });

                let mir_op = match b.op {
                    HirBinaryOp::Add => MirBinOp::Add,
                    HirBinaryOp::Sub => MirBinOp::Sub,
                    HirBinaryOp::Mul => MirBinOp::Mul,
                    HirBinaryOp::Div => MirBinOp::Div,
                    HirBinaryOp::Mod => MirBinOp::Mod,
                    HirBinaryOp::And => MirBinOp::And,
                    HirBinaryOp::Or => MirBinOp::Or,
                    HirBinaryOp::Eq => MirBinOp::Eq,
                    HirBinaryOp::Ne => MirBinOp::Ne,
                    HirBinaryOp::Lt => MirBinOp::Lt,
                    HirBinaryOp::Le => MirBinOp::Le,
                    HirBinaryOp::Gt => MirBinOp::Gt,
                    HirBinaryOp::Ge => MirBinOp::Ge,
                    _ => MirBinOp::Add,
                };

                all_insts.push(MirInst::Assign {
                    dest: result_local,
                    value: MirValue::BinOp { op: mir_op, lhs: lhs_local, rhs: rhs_local },
                    span: dummy_span(),
                });

                (MirValue::Local(result_local), all_insts)
            }
            HirExpr::Call(c) => {
                let mut all_insts = Vec::new();
                let mut args = Vec::new();

                for arg in &c.args {
                    let (val, arg_insts) = self.lower_expr(arg);
                    all_insts.extend(arg_insts);
                    let local = self.alloc_local();
                    all_insts.push(MirInst::Assign { dest: local, value: val, span: dummy_span() });
                    args.push(local);
                }

                let callee_name = match &*c.callee {
                    HirExpr::Ident(name, _) => name.clone(),
                    _ => "unknown".to_string(),
                };

                let dest = self.alloc_local();
                all_insts.push(MirInst::Call {
                    dest: Some(dest),
                    callee: callee_name,
                    args,
                    span: dummy_span(),
                });

                (MirValue::Local(dest), all_insts)
            }
            HirExpr::JsxElement(elem) => {
                let mut all_insts = Vec::new();

                let tag_local = self.alloc_local();
                all_insts.push(MirInst::Assign {
                    dest: tag_local,
                    value: MirValue::String(elem.tag.clone()),
                    span: dummy_span(),
                });

                let attrs_local = self.alloc_local();
                let mut attr_fields = Vec::new();
                for (name, val) in &elem.attrs {
                    let (attr_val, attr_insts) = self.lower_expr(val);
                    all_insts.extend(attr_insts);
                    let attr_local = self.alloc_local();
                    all_insts.push(MirInst::Assign { dest: attr_local, value: attr_val, span: dummy_span() });
                    attr_fields.push((name.clone(), attr_local));
                }
                all_insts.push(MirInst::Assign {
                    dest: attrs_local,
                    value: MirValue::StructInit { name: "Attrs".into(), fields: attr_fields },
                    span: dummy_span(),
                });

                let children_local = self.alloc_local();
                let mut child_values = Vec::new();
                for child in &elem.children {
                    let (child_val, child_insts) = self.lower_expr(child);
                    all_insts.extend(child_insts);
                    let child_local = self.alloc_local();
                    all_insts.push(MirInst::Assign { dest: child_local, value: child_val, span: dummy_span() });
                    child_values.push(child_local);
                }

                let result_local = self.alloc_local();
                all_insts.push(MirInst::Call {
                    dest: Some(result_local),
                    callee: "h".into(),
                    args: vec![tag_local, attrs_local, children_local],
                    span: dummy_span(),
                });

                (MirValue::Local(result_local), all_insts)
            }
            HirExpr::Unary(u) => {
                let (val, val_insts) = self.lower_expr(&u.expr);
                let val_local = self.alloc_local();
                let result_local = self.alloc_local();
                let mut all_insts = val_insts;
                all_insts.push(MirInst::Assign { dest: val_local, value: val, span: dummy_span() });

                let mir_op = match u.op {
                    HirUnaryOp::Neg => MirUnOp::Neg,
                    HirUnaryOp::Not => MirUnOp::Not,
                };

                all_insts.push(MirInst::Assign {
                    dest: result_local,
                    value: MirValue::UnOp { op: mir_op, expr: val_local },
                    span: dummy_span(),
                });

                (MirValue::Local(result_local), all_insts)
            }
            HirExpr::Member(m) => {
                let (obj_val, obj_insts) = self.lower_expr(&m.object);
                let obj_local = self.alloc_local();
                let result_local = self.alloc_local();
                let mut all_insts = obj_insts;
                all_insts.push(MirInst::Assign { dest: obj_local, value: obj_val, span: dummy_span() });
                all_insts.push(MirInst::Assign {
                    dest: result_local,
                    value: MirValue::GetField { object: obj_local, name: String::new(), field: m.field.clone() },
                    span: dummy_span(),
                });
                (MirValue::Local(result_local), all_insts)
            }
            _ => (MirValue::Int(0), vec![]),
        }
    }

    fn find_or_alloc_local(&mut self, name: &str, locals: &mut Vec<MirLocal>) -> usize {
        if let Some(pos) = locals.iter().position(|l| l.name == name) {
            pos
        } else {
            let id = self.alloc_local();
            locals.push(MirLocal { name: name.to_string(), ty: MirType::I64 });
            id
        }
    }
}

fn convert_type(ty: &TypeInfo) -> MirType {
    match ty {
        TypeInfo::I32 => MirType::I32,
        TypeInfo::I64 | TypeInfo::U64 => MirType::I64,
        TypeInfo::U32 => MirType::I32,
        TypeInfo::F32 => MirType::F32,
        TypeInfo::F64 => MirType::F64,
        TypeInfo::Bool => MirType::Bool,
        TypeInfo::String | TypeInfo::Char => MirType::String,
        TypeInfo::Void | TypeInfo::Node => MirType::Void,
        TypeInfo::Array(_) | TypeInfo::Optional(_) => MirType::I64,
        TypeInfo::Fn(_) => MirType::I64,
        TypeInfo::Struct(s) => MirType::Named(s.name.clone()),
        TypeInfo::Enum(e) => MirType::Named(e.name.clone()),
        TypeInfo::Generic(s) => MirType::Named(s.clone()),
        _ => MirType::I64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = RakitToBrakBridge::new();
        assert_eq!(bridge.next_local, 0);
    }

    #[test]
    fn test_convert_type() {
        assert!(matches!(convert_type(&TypeInfo::I32), MirType::I32));
        assert!(matches!(convert_type(&TypeInfo::Bool), MirType::Bool));
        assert!(matches!(convert_type(&TypeInfo::String), MirType::String));
    }
}
