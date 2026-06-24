use rakit_ir_ast as ast;
use crate::hir::*;
use crate::ty::*;
use super::{HirLower, lower_binary_op, lower_unary_op};

impl HirLower {
    pub fn lower_expr(&mut self, expr: &ast::Expr) -> HirExpr {
        match expr {
            ast::Expr::Literal(lit) => self.lower_literal(lit),
            ast::Expr::Ident(name) => HirExpr::Ident(name.clone(), TypeInfo::Infer),
            ast::Expr::Binary(op, lhs, rhs) => HirExpr::Binary(HirBinary {
                op: lower_binary_op(*op),
                lhs: Box::new(self.lower_expr(lhs)),
                rhs: Box::new(self.lower_expr(rhs)),
                ty: TypeInfo::Infer,
            }),
            ast::Expr::Unary(op, expr) => HirExpr::Unary(HirUnary {
                op: lower_unary_op(*op),
                expr: Box::new(self.lower_expr(expr)),
                ty: TypeInfo::Infer,
            }),
            ast::Expr::Assign { target, value } => HirExpr::Assign(HirAssign {
                target: Box::new(self.lower_expr(target)),
                value: Box::new(self.lower_expr(value)),
                ty: TypeInfo::Infer,
            }),
            ast::Expr::Ternary(cond, then_expr, else_expr) => HirExpr::Ternary(HirTernary {
                condition: Box::new(self.lower_expr(cond)),
                then_expr: Box::new(self.lower_expr(then_expr)),
                else_expr: Box::new(self.lower_expr(else_expr)),
                ty: TypeInfo::Infer,
            }),
            ast::Expr::Call(callee, args) => HirExpr::Call(HirCall {
                callee: Box::new(self.lower_expr(callee)),
                args: args.iter().map(|a| self.lower_expr(a)).collect(),
                ty: TypeInfo::Infer,
            }),
            ast::Expr::Member(obj, field) => HirExpr::Member(HirMember {
                object: Box::new(self.lower_expr(obj)),
                field: field.clone(),
                ty: TypeInfo::Infer,
            }),
            ast::Expr::Index(obj, index) => HirExpr::Index(HirIndex {
                object: Box::new(self.lower_expr(obj)),
                index: Box::new(self.lower_expr(index)),
                ty: TypeInfo::Infer,
            }),
            ast::Expr::StructInit(name, fields) => {
                let hir_fields = fields.iter().map(|f| HirStructInitField {
                    name: f.name.clone(),
                    value: self.lower_expr(&f.value),
                    spread: f.spread,
                }).collect();
                HirExpr::StructInit(HirStructInit {
                    name: name.clone(),
                    fields: hir_fields,
                    ty: TypeInfo::Infer,
                })
            }
            ast::Expr::Array(items) => {
                let hir_items: Vec<HirExpr> = items.iter().map(|i| self.lower_expr(i)).collect();
                HirExpr::Array(hir_items, TypeInfo::Infer)
            }
            ast::Expr::JsxElement(elem) => {
                self.lower_jsx_element(elem)
            }
            ast::Expr::JsxFragment(frag) => {
                self.lower_jsx_fragment(frag)
            }
            ast::Expr::BlockExpr(block) => HirExpr::Block(self.lower_block(block)),
        }
    }

    fn lower_literal(&mut self, lit: &ast::Literal) -> HirExpr {
        match lit {
            ast::Literal::Number(n) => HirExpr::Number(*n, TypeInfo::F64),
            ast::Literal::String(s) => HirExpr::String(s.clone(), TypeInfo::String),
            ast::Literal::Bool(b) => HirExpr::Bool(*b, TypeInfo::Bool),
            ast::Literal::Null => HirExpr::Null(TypeInfo::Optional(Box::new(TypeInfo::Infer))),
            ast::Literal::Char(c) => HirExpr::String(c.to_string(), TypeInfo::Char),
        }
    }

    pub fn lower_block(&mut self, block: &ast::Block) -> HirBlock {
        let stmts: Vec<HirStmt> = block.stmts.iter()
            .filter_map(|s| self.lower_stmt(s))
            .collect();
        HirBlock { stmts }
    }
}
