use rakit_ir_hir::hir::*;

#[derive(Debug, Clone)]
pub enum DocItemKind {
    Function,
    Component,
    Struct,
    Enum,
    TypeAlias,
}

#[derive(Debug, Clone)]
pub struct DocParam {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DocItem {
    pub name: String,
    pub kind: DocItemKind,
    pub description: String,
    pub params: Vec<DocParam>,
    pub returns: Option<String>,
    pub examples: Vec<String>,
    pub source_location: (String, usize),
}

pub fn extract_docs_from_hir(program: &HirProgram) -> Vec<DocItem> {
    let mut docs = Vec::new();

    for item in &program.items {
        match item {
            HirItem::Function(f) => {
                let mut params = Vec::new();
                for p in &f.params {
                    params.push(DocParam {
                        name: p.name.clone(),
                        description: format!("Parameter bertipe {:?}", p.ty),
                    });
                }
                docs.push(DocItem {
                    name: f.name.clone(),
                    kind: DocItemKind::Function,
                    description: format!("Fungsi `{}`", f.name),
                    params,
                    returns: Some(format!("{:?}", f.return_ty)),
                    examples: vec![],
                    source_location: (String::new(), 0),
                });
            }
            HirItem::Component(c) => {
                let mut params = Vec::new();
                params.push(DocParam {
                    name: c.props_param.name.clone(),
                    description: format!("Props bertipe {:?}", c.props_param.ty),
                });
                docs.push(DocItem {
                    name: c.name.clone(),
                    kind: DocItemKind::Component,
                    description: format!("Komponen `{}`", c.name),
                    params,
                    returns: None,
                    examples: vec![],
                    source_location: (String::new(), 0),
                });
            }
            HirItem::Struct(s) => {
                let mut params = Vec::new();
                for f in &s.fields {
                    params.push(DocParam {
                        name: f.name.clone(),
                        description: format!("{:?}", f.ty),
                    });
                }
                docs.push(DocItem {
                    name: s.name.clone(),
                    kind: DocItemKind::Struct,
                    description: format!("Struktur `{}`", s.name),
                    params,
                    returns: None,
                    examples: vec![],
                    source_location: (String::new(), 0),
                });
            }
            HirItem::Enum(e) => {
                docs.push(DocItem {
                    name: e.name.clone(),
                    kind: DocItemKind::Enum,
                    description: format!("Enum `{}` dengan {} varian", e.name, e.variants.len()),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    source_location: (String::new(), 0),
                });
            }
            HirItem::TypeAlias(t) => {
                docs.push(DocItem {
                    name: t.name.clone(),
                    kind: DocItemKind::TypeAlias,
                    description: format!("Alias tipe ke {:?}", t.ty),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    source_location: (String::new(), 0),
                });
            }
            _ => {}
        }
    }

    docs
}
