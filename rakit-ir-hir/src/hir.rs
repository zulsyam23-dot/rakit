use rakit_core::Span;
use crate::ty::*;

#[derive(Debug, Clone)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
}

#[derive(Debug, Clone)]
pub struct HirStruct {
    pub name: String,
    pub fields: Vec<FieldType>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HirEnum {
    pub name: String,
    pub variants: Vec<VariantType>,
}

#[derive(Debug, Clone)]
pub struct HirTypeAlias {
    pub name: String,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct HirImport {
    pub module: String,
    pub names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HirExport {
    pub item_name: String,
}

#[derive(Debug, Clone)]
pub enum HirItem {
    Function(HirFunction),
    Component(HirComponent),
    Struct(HirStruct),
    Enum(HirEnum),
    TypeAlias(HirTypeAlias),
    Import(HirImport),
    Export(HirExport),
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_ty: TypeInfo,
    pub body: HirBlock,
    pub type_params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HirComponent {
    pub name: String,
    pub props_param: HirParam,
    pub hook_calls: Vec<HirHookCall>,
    pub body_stmts: Vec<HirStmt>,
    pub render: HirExpr,
}

#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub ty: TypeInfo,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirHookCall {
    pub kind: HookKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HookKind {
    State {
        state_var: String,
        setter_var: String,
        initial: Box<HirExpr>,
        ty: TypeInfo,
    },
    Effect {
        callback: Box<HirExpr>,
        deps: Vec<HirExpr>,
    },
    Memo {
        result_var: String,
        callback: Box<HirExpr>,
        deps: Vec<HirExpr>,
        ty: TypeInfo,
    },
}

#[derive(Debug, Clone)]
pub enum HirStmt {
    Let(HirLet),
    Expr(HirExpr),
    If(HirIf),
    While(HirWhile),
    Match(HirMatch),
    Return(Option<HirExpr>),
    Block(HirBlock),
    Break,
    Continue,
    Try(HirTry),
    Throw(HirExpr),
}

#[derive(Debug, Clone)]
pub struct HirLet {
    pub name: String,
    pub mutable: bool,
    pub ty: TypeInfo,
    pub value: HirExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirIf {
    pub condition: HirExpr,
    pub then_block: HirBlock,
    pub else_block: Option<HirBlock>,
}

#[derive(Debug, Clone)]
pub struct HirWhile {
    pub condition: HirExpr,
    pub body: HirBlock,
}

#[derive(Debug, Clone)]
pub struct HirMatch {
    pub expr: HirExpr,
    pub arms: Vec<HirMatchArm>,
}

#[derive(Debug, Clone)]
pub struct HirMatchArm {
    pub pattern: HirPattern,
    pub body: HirExpr,
}

#[derive(Debug, Clone)]
pub enum HirPattern {
    Wildcard,
    Literal(HirLiteral),
    Ident(String),
    Struct { name: String, fields: Vec<(String, HirPattern)> },
    Enum { name: String, variant: String, fields: Vec<HirPattern> },
}

#[derive(Debug, Clone)]
pub struct HirTry {
    pub try_block: HirBlock,
    pub catch_var: String,
    pub catch_block: HirBlock,
}

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
}

#[derive(Debug, Clone)]
pub enum HirExpr {
    Number(f64, TypeInfo),
    String(String, TypeInfo),
    Bool(bool, TypeInfo),
    Null(TypeInfo),
    Ident(String, TypeInfo),
    Binary(HirBinary),
    Unary(HirUnary),
    Assign(HirAssign),
    Ternary(HirTernary),
    Call(HirCall),
    Member(HirMember),
    Index(HirIndex),
    Array(Vec<HirExpr>, TypeInfo),
    StructInit(HirStructInit),
    JsxElement(Box<HirJsxElement>),
    HookState(Box<HirHookState>),
    HookEffect(HirHookEffect),
    HookMemo(HirHookMemo),
    Block(HirBlock),
}

#[derive(Debug, Clone)]
pub struct HirBinary {
    pub op: HirBinaryOp,
    pub lhs: Box<HirExpr>,
    pub rhs: Box<HirExpr>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub enum HirBinaryOp {
    Add, Sub, Mul, Div, Mod,
    And, Or, Eq, Ne, Lt, Gt, Le, Ge,
    Concat, NullCoalescing,
}

#[derive(Debug, Clone)]
pub struct HirUnary {
    pub op: HirUnaryOp,
    pub expr: Box<HirExpr>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub enum HirUnaryOp {
    Neg, Not,
}

#[derive(Debug, Clone)]
pub struct HirAssign {
    pub target: Box<HirExpr>,
    pub value: Box<HirExpr>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct HirTernary {
    pub condition: Box<HirExpr>,
    pub then_expr: Box<HirExpr>,
    pub else_expr: Box<HirExpr>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct HirCall {
    pub callee: Box<HirExpr>,
    pub args: Vec<HirExpr>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct HirMember {
    pub object: Box<HirExpr>,
    pub field: String,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct HirIndex {
    pub object: Box<HirExpr>,
    pub index: Box<HirExpr>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct HirStructInit {
    pub name: String,
    pub fields: Vec<HirStructInitField>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct HirStructInitField {
    pub name: String,
    pub value: HirExpr,
    pub spread: bool,
}

#[derive(Debug, Clone)]
pub struct HirJsxElement {
    pub tag: String,
    pub attrs: Vec<(String, HirExpr)>,
    pub children: Vec<HirExpr>,
    pub ty: TypeInfo,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirHookState {
    pub name: String,
    pub state_var: String,
    pub setter_var: String,
    pub initial: Box<HirExpr>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct HirHookEffect {
    pub callback: Box<HirExpr>,
    pub deps: Vec<HirExpr>,
}

#[derive(Debug, Clone)]
pub struct HirHookMemo {
    pub result_var: String,
    pub callback: Box<HirExpr>,
    pub deps: Vec<HirExpr>,
    pub ty: TypeInfo,
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}
