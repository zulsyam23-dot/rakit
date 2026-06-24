pub mod checker;
pub mod infer;
pub mod unify;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    I32, I64, U32, U64, F32, F64,
    Bool, String, Char, Void, Node,
    Array(Box<TypeInfo>),
    Optional(Box<TypeInfo>),
    Fn(FnType),
    Struct(StructType),
    Enum(EnumType),
    Tuple(Vec<TypeInfo>),
    Generic(String),
    TypeParam(usize),
    Infer,
    Error,
}

impl TypeInfo {
    pub fn is_numeric(&self) -> bool {
        matches!(self, TypeInfo::I32 | TypeInfo::I64 | TypeInfo::U32
            | TypeInfo::U64 | TypeInfo::F32 | TypeInfo::F64)
    }

    pub fn is_comparable_to(&self, other: &TypeInfo) -> bool {
        match (self, other) {
            (a, b) if a == b => true,
            (TypeInfo::I32, TypeInfo::I64)
            | (TypeInfo::I64, TypeInfo::I32)
            | (TypeInfo::F32, TypeInfo::F64)
            | (TypeInfo::F64, TypeInfo::F32) => true,
            _ => false,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, TypeInfo::Error)
    }

    pub fn is_infer(&self) -> bool {
        matches!(self, TypeInfo::Infer)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnType {
    pub params: Vec<TypeInfo>,
    pub ret: Box<TypeInfo>,
}

impl FnType {
    pub fn new(params: Vec<TypeInfo>, ret: TypeInfo) -> Self {
        FnType { params, ret: Box::new(ret) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<FieldType>,
    pub generics: Vec<TypeInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldType {
    pub name: String,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<VariantType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariantType {
    pub name: String,
    pub fields: Vec<TypeInfo>,
}

pub fn type_widen(a: &TypeInfo, b: &TypeInfo) -> TypeInfo {
    use TypeInfo::*;
    match (a, b) {
        (F64, _) | (_, F64) => F64,
        (F32, _) | (_, F32) => F32,
        (I64, _) | (_, I64) => I64,
        (U64, _) | (_, U64) => U64,
        (I32, I32) => I32,
        (U32, U32) => U32,
        _ => F64,
    }
}
