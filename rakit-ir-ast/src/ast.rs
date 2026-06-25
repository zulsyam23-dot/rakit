use rakit_core::Span;
use crate::ty::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Item {
    Function(FnDef),
    Component(ComponentDef),
    Struct(StructDef),
    Enum(EnumDef),
    TypeAlias(TypeAlias),
    Import(Import),
    Export(Export),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<FnParam>,
    pub return_ty: Option<Type>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FnParam {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentDef {
    pub name: String,
    pub props: Vec<FnParam>,
    pub body: ComponentBody,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentBody {
    pub statements: Vec<Stmt>,
    pub render: Expr,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub generics: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeAlias {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Import {
    pub module: String,
    pub names: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Export {
    pub item_name: String,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Stmt {
    Let(LetDef),
    Expr(Expr, Span),
    If(IfStmt),
    While(WhileStmt),
    Match(MatchStmt),
    Return(Option<Expr>, Span),
    Block(Block),
    Break(Span),
    Continue(Span),
    Try(TryStmt),
    Throw(Expr, Span),
    HookState(HookStateDef),
    FnDef(FnDef),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LetDef {
    pub name: String,
    pub mutable: bool,
    pub ty: Option<Type>,
    pub value: Expr,
    pub pattern: Option<Pattern>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HookStateDef {
    pub state_var: String,
    pub setter_var: String,
    pub mutable: bool,
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
    pub label: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchStmt {
    pub expr: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    Ident(String),
    Struct { name: String, fields: Vec<(String, Pattern)> },
    Enum { name: String, variant: String, fields: Vec<Pattern> },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TryStmt {
    pub try_block: Block,
    pub catch_var: String,
    pub catch_block: Block,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Assign { target: Box<Expr>, value: Box<Expr> },
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Member(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    StructInit(String, Vec<StructInitField>),
    Object(Vec<StructInitField>),
    Array(Vec<Expr>),
    Spread(Box<Expr>),
    ArrowFn(ArrowFn),
    JsxElement(Box<JsxElement>),
    JsxFragment(Box<JsxFragment>),
    BlockExpr(Block),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Char(char),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructInitField {
    pub name: String,
    pub value: Expr,
    pub spread: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArrowFn {
    pub params: Vec<FnParam>,
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    And, Or,
    Eq, Ne, Lt, Gt, Le, Ge,
    Concat,
    NullCoalescing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UnaryOp {
    Neg, Not,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsxElement {
    pub tag: String,
    pub attrs: Vec<JsxAttr>,
    pub children: Vec<JsxChild>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsxFragment {
    pub children: Vec<JsxChild>,
    pub span: Span,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum JsxChild {
    Element(Box<JsxElement>),
    Fragment(Box<JsxFragment>),
    Expr(Expr),
    Text(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum JsxAttr {
    Literal { name: String, value: String, span: Span },
    Expr { name: String, value: Expr, span: Span },
    Spread(Expr, Span),
}
