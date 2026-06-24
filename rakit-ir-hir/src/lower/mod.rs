use rakit_core::{Span, Diagnostic};
use rakit_ir_ast as ast;
use crate::hir::*;
use crate::ty::*;

pub mod expr;
pub mod stmt;
pub mod jsx;
pub mod pattern;

pub struct HirLower {
    next_id: usize,
    diagnostics: Vec<Diagnostic>,
}

impl HirLower {
    pub fn new() -> Self {
        HirLower { next_id: 0, diagnostics: Vec::new() }
    }

    pub fn gen_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn lower_program(&mut self, program: &ast::Program) -> std::result::Result<HirProgram, Vec<Diagnostic>> {
        let mut items = Vec::new();
        for item in &program.items {
            match self.lower_item(item) {
                Ok(hir_item) => items.push(hir_item),
                Err(diag) => self.diagnostics.push(diag),
            }
        }
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }
        Ok(HirProgram { items })
    }

    pub fn lower_item(&mut self, item: &ast::Item) -> Result<HirItem> {
        match item {
            ast::Item::Function(f) => self.lower_function(f),
            ast::Item::Component(c) => self.lower_component(c),
            ast::Item::Struct(s) => self.lower_struct(s),
            ast::Item::Enum(e) => self.lower_enum(e),
            ast::Item::TypeAlias(t) => self.lower_type_alias(t),
            ast::Item::Import(i) => self.lower_import(i),
            ast::Item::Export(e) => self.lower_export(e),
        }
    }

    fn lower_function(&mut self, f: &ast::FnDef) -> Result<HirItem> {
        let mut params = Vec::new();
        for p in &f.params {
            params.push(HirParam {
                name: p.name.clone(),
                ty: lower_type(&p.ty),
                span: p.span,
            });
        }
        let return_ty = f.return_ty.as_ref().map(|t| lower_type(t)).unwrap_or(TypeInfo::Void);
        let body = self.lower_block(&f.body);
        Ok(HirItem::Function(HirFunction {
            name: f.name.clone(),
            params,
            return_ty,
            body,
            type_params: Vec::new(),
        }))
    }

    fn lower_component(&mut self, c: &ast::ComponentDef) -> Result<HirItem> {
        let props_param = if c.props.is_empty() {
            HirParam {
                name: "props".into(),
                ty: TypeInfo::Struct(StructType {
                    name: "{}".into(),
                    fields: Vec::new(),
                    generics: Vec::new(),
                }),
                span: Span::empty(0),
            }
        } else {
            let fields: Vec<FieldType> = c.props.iter().map(|p| FieldType {
                name: p.name.clone(),
                ty: lower_type(&p.ty),
            }).collect();
            HirParam {
                name: "props".into(),
                ty: TypeInfo::Struct(StructType {
                    name: format!("Props_{}", c.name),
                    fields,
                    generics: Vec::new(),
                }),
                span: Span::empty(0),
            }
        };

        let mut hook_calls = Vec::new();
        let mut body_stmts = Vec::new();

        // Buat let binding untuk setiap prop agar nama prop bisa diakses langsung
        // (seperti Svelte — nama prop langsung available di scope komponen)
        for p in &c.props {
            body_stmts.push(HirStmt::Let(HirLet {
                name: p.name.clone(),
                mutable: false,
                ty: lower_type(&p.ty),
                value: HirExpr::Member(HirMember {
                    object: Box::new(HirExpr::Ident("props".into(), TypeInfo::Infer)),
                    field: p.name.clone(),
                    ty: lower_type(&p.ty),
                }),
                span: p.span,
            }));
        }

        for stmt in &c.body.statements {
            self.lower_stmt_in_component(stmt, &mut hook_calls, &mut body_stmts);
        }
        let render = self.lower_expr(&c.body.render);

        Ok(HirItem::Component(HirComponent {
            name: c.name.clone(),
            props_param,
            hook_calls,
            body_stmts,
            render,
        }))
    }

    fn lower_struct(&mut self, s: &ast::StructDef) -> Result<HirItem> {
        let fields: Vec<FieldType> = s.fields.iter().map(|f| FieldType {
            name: f.name.clone(),
            ty: lower_type(&f.ty),
        }).collect();
        Ok(HirItem::Struct(HirStruct {
            name: s.name.clone(),
            fields,
            generics: s.generics.clone(),
        }))
    }

    fn lower_enum(&mut self, e: &ast::EnumDef) -> Result<HirItem> {
        let variants = e.variants.iter().map(|v| VariantType {
            name: v.name.clone(),
            fields: v.fields.iter().map(lower_type).collect(),
        }).collect();
        Ok(HirItem::Enum(HirEnum {
            name: e.name.clone(),
            variants,
        }))
    }

    fn lower_type_alias(&mut self, t: &ast::TypeAlias) -> Result<HirItem> {
        Ok(HirItem::TypeAlias(HirTypeAlias {
            name: t.name.clone(),
            ty: lower_type(&t.ty),
        }))
    }

    fn lower_import(&mut self, i: &ast::Import) -> Result<HirItem> {
        Ok(HirItem::Import(HirImport {
            module: i.module.clone(),
            names: i.names.clone(),
        }))
    }

    fn lower_export(&mut self, e: &ast::Export) -> Result<HirItem> {
        Ok(HirItem::Export(HirExport {
            item_name: e.item_name.clone(),
        }))
    }
}

fn lower_type(ty: &ast::Type) -> TypeInfo {
    use ast::Type as AstTy;
    match ty {
        AstTy::Named(name) => match name.as_str() {
            "I32" => TypeInfo::I32,
            "I64" => TypeInfo::I64,
            "U32" => TypeInfo::U32,
            "U64" => TypeInfo::U64,
            "F32" => TypeInfo::F32,
            "F64" => TypeInfo::F64,
            "Bool" | "bool" => TypeInfo::Bool,
            "String" | "string" => TypeInfo::String,
            "Char" | "char" => TypeInfo::Char,
            "Void" | "void" => TypeInfo::Void,
            "Node" | "node" => TypeInfo::Node,
            "Int" => TypeInfo::I32,
            "Float" => TypeInfo::F64,
            _ => TypeInfo::Generic(name.clone()),
        },
        AstTy::Generic(g) => TypeInfo::Generic(g.name.clone()),
        AstTy::Struct(fields) => TypeInfo::Struct(StructType {
            name: "anonymous".into(),
            fields: fields.iter().map(|f| FieldType {
                name: f.name.clone(),
                ty: lower_type(&f.ty),
            }).collect(),
            generics: Vec::new(),
        }),
        AstTy::Tuple(tys) => TypeInfo::Tuple(tys.iter().map(lower_type).collect()),
        AstTy::Array(inner) => TypeInfo::Array(Box::new(lower_type(inner))),
        AstTy::Fn(params, ret) => TypeInfo::Fn(FnType {
            params: params.iter().map(lower_type).collect(),
            ret: Box::new(lower_type(ret)),
        }),
        AstTy::Optional(inner) => TypeInfo::Optional(Box::new(lower_type(inner))),
        AstTy::Infer => TypeInfo::Infer,
    }
}

fn lower_binary_op(op: ast::BinaryOp) -> HirBinaryOp {
    use ast::BinaryOp::*;
    match op {
        Add => HirBinaryOp::Add, Sub => HirBinaryOp::Sub,
        Mul => HirBinaryOp::Mul, Div => HirBinaryOp::Div,
        Mod => HirBinaryOp::Mod,
        And => HirBinaryOp::And, Or => HirBinaryOp::Or,
        Eq => HirBinaryOp::Eq, Ne => HirBinaryOp::Ne,
        Lt => HirBinaryOp::Lt, Gt => HirBinaryOp::Gt,
        Le => HirBinaryOp::Le, Ge => HirBinaryOp::Ge,
        Concat => HirBinaryOp::Concat,
    }
}

fn lower_unary_op(op: ast::UnaryOp) -> HirUnaryOp {
    match op {
        ast::UnaryOp::Neg => HirUnaryOp::Neg,
        ast::UnaryOp::Not => HirUnaryOp::Not,
    }
}

pub type Result<T> = std::result::Result<T, Diagnostic>;
