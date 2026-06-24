use rakit_ir_ast as ast;
use crate::hir::*;
use crate::ty::*;
use super::HirLower;

impl HirLower {
    /// Desugar JSX element → h() call expression
    ///
    /// Transform:
    ///   <Nama tag={expr}> child1 child2 </Nama>
    ///     → h("Nama", { attrs }, [child1, child2])
    ///
    ///   <tag attr="val"> child </tag>
    ///     → h("tag", { attr: "val" }, [child])
    pub fn lower_jsx_element(&mut self, jsx: &ast::JsxElement) -> HirExpr {
        let attrs = self.lower_jsx_attrs(&jsx.attrs);
        let children: Vec<HirExpr> = jsx.children.iter()
            .map(|c| self.lower_jsx_child(c))
            .collect();

        let attrs_struct = HirExpr::StructInit(HirStructInit {
            name: "Attrs".into(),
            fields: attrs.iter().map(|(name, expr)| HirStructInitField {
                name: name.clone(),
                value: expr.clone(),
                spread: false,
            }).collect(),
            ty: TypeInfo::Struct(StructType {
                name: "Attrs".into(),
                fields: attrs.iter().map(|(name, _)| FieldType {
                    name: name.clone(),
                    ty: TypeInfo::Infer,
                }).collect(),
                generics: Vec::new(),
            }),
        });

        HirExpr::Call(HirCall {
            callee: Box::new(HirExpr::Ident("h".into(), TypeInfo::Fn(FnType {
                params: vec![TypeInfo::String, TypeInfo::Infer, TypeInfo::Array(Box::new(TypeInfo::Node))],
                ret: Box::new(TypeInfo::Node),
            }))),
            args: vec![
                HirExpr::String(jsx.tag.clone(), TypeInfo::String),
                attrs_struct,
                HirExpr::Array(children, TypeInfo::Array(Box::new(TypeInfo::Node))),
            ],
            ty: TypeInfo::Node,
        })
    }

    /// Desugar JSX fragment → h("fragment", {}, [children])
    pub fn lower_jsx_fragment(&mut self, frag: &ast::JsxFragment) -> HirExpr {
        let children: Vec<HirExpr> = frag.children.iter()
            .map(|c| self.lower_jsx_child(c))
            .collect();

        HirExpr::Call(HirCall {
            callee: Box::new(HirExpr::Ident("h".into(), TypeInfo::Fn(FnType {
                params: vec![TypeInfo::String, TypeInfo::Infer, TypeInfo::Array(Box::new(TypeInfo::Node))],
                ret: Box::new(TypeInfo::Node),
            }))),
            args: vec![
                HirExpr::String("fragment".into(), TypeInfo::String),
                HirExpr::StructInit(HirStructInit {
                    name: "Attrs".into(),
                    fields: Vec::new(),
                    ty: TypeInfo::Struct(StructType {
                        name: "Attrs".into(),
                        fields: Vec::new(),
                        generics: Vec::new(),
                    }),
                }),
                HirExpr::Array(children, TypeInfo::Array(Box::new(TypeInfo::Node))),
            ],
            ty: TypeInfo::Node,
        })
    }

    /// Desugar JSX attributes into Vec<(name, expr)>
    fn lower_jsx_attrs(&mut self, attrs: &[ast::JsxAttr]) -> Vec<(String, HirExpr)> {
        attrs.iter().map(|attr| {
            match attr {
                ast::JsxAttr::Literal { name, value, .. } => {
                    (name.clone(), HirExpr::String(value.clone(), TypeInfo::String))
                }
                ast::JsxAttr::Expr { name, value, .. } => {
                    (name.clone(), self.lower_expr(value))
                }
                ast::JsxAttr::Spread(_, _) => {
                    (String::new(), HirExpr::Null(TypeInfo::Infer))
                }
            }
        }).collect()
    }

    /// Desugar JSX child → HirExpr
    fn lower_jsx_child(&mut self, child: &ast::JsxChild) -> HirExpr {
        match child {
            ast::JsxChild::Element(elem) => self.lower_jsx_element(elem),
            ast::JsxChild::Fragment(frag) => self.lower_jsx_fragment(frag),
            ast::JsxChild::Expr(expr) => self.lower_expr(expr),
            ast::JsxChild::Text(text) => {
                HirExpr::String(text.clone(), TypeInfo::String)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rakit_ir_ast as ast;

    #[test]
    fn test_jsx_element_desugaring() {
        let jsx = ast::JsxElement {
            tag: "div".into(),
            attrs: vec![
                ast::JsxAttr::Literal {
                    name: "className".into(),
                    value: "app".into(),
                    span: Default::default(),
                },
            ],
            children: vec![
                ast::JsxChild::Text("Halo".into()),
            ],
            span: Default::default(),
        };

        let mut lower = HirLower::new();
        let result = lower.lower_jsx_element(&jsx);

        match result {
            HirExpr::Call(call) => {
                assert!(matches!(&*call.callee, HirExpr::Ident(name, _) if name == "h"));
                assert_eq!(call.args.len(), 3);
            }
            _ => panic!("Expected HirExpr::Call"),
        }
    }

    #[test]
    fn test_jsx_fragment_desugaring() {
        let frag = ast::JsxFragment {
            children: vec![
                ast::JsxChild::Text("a".into()),
                ast::JsxChild::Text("b".into()),
            ],
            span: Default::default(),
        };

        let mut lower = HirLower::new();
        let result = lower.lower_jsx_fragment(&frag);

        match result {
            HirExpr::Call(call) => {
                if let HirExpr::String(tag, _) = &call.args[0] {
                    assert_eq!(tag, "fragment");
                } else {
                    panic!("Expected string tag");
                }
            }
            _ => panic!("Expected HirExpr::Call"),
        }
    }
}
