pub mod c_bindings;
pub mod python_bindings;
pub mod rust_bindings;
pub mod abi;

use rakit_ir_hir::ty::TypeInfo;

#[derive(Debug, Clone)]
pub struct RakitExport {
    pub name: String,
    pub params: Vec<RakitParam>,
    pub return_ty: TypeInfo,
    pub is_async: bool,
}

impl RakitExport {
    pub fn c_name(&self) -> String {
        format!("rakit_{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct RakitParam {
    pub name: String,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct RakitStruct {
    pub name: String,
    pub fields: Vec<RakitField>,
}

#[derive(Debug, Clone)]
pub struct RakitField {
    pub name: String,
    pub ty: TypeInfo,
}
