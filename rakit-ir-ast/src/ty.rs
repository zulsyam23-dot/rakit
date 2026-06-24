use rakit_core::Span;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Type {
    Named(String),
    Generic(GenericType),
    Struct(Vec<StructField>),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Fn(Vec<Type>, Box<Type>),
    Optional(Box<Type>),
    Infer,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GenericType {
    pub name: String,
    pub params: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}
