/// Brak IR type definitions for Rakit compiler.
/// Intermediate representation between HIR and code generation.

#[derive(Debug, Clone)]
pub enum BrakItem {
    Function(BrakFnDef),
    Struct(BrakStructDef),
    Enum(BrakEnumDef),
}

#[derive(Debug, Clone)]
pub struct BrakFnDef {
    pub name: String,
    pub params: Vec<BrakParam>,
    pub return_ty: Option<BrakTy>,
    pub body: BrakBlock,
    pub is_component: bool,
    pub hook_calls: Vec<BrakHookCall>,
}

#[derive(Debug, Clone)]
pub struct BrakParam {
    pub name: String,
    pub ty: BrakTy,
}

#[derive(Debug, Clone)]
pub struct BrakStructDef {
    pub name: String,
    pub fields: Vec<BrakStructField>,
}

#[derive(Debug, Clone)]
pub struct BrakStructField {
    pub name: String,
    pub ty: BrakTy,
}

#[derive(Debug, Clone)]
pub struct BrakEnumDef {
    pub name: String,
    pub variants: Vec<BrakEnumVariant>,
}

#[derive(Debug, Clone)]
pub struct BrakEnumVariant {
    pub name: String,
    pub fields: Vec<BrakTy>,
}

#[derive(Debug, Clone)]
pub struct BrakBlock {
    pub stmts: Vec<BrakStmt>,
}

#[derive(Debug, Clone)]
pub enum BrakStmt {
    Let(BrakLet),
    Expr(BrakExpr),
    If(BrakIf),
    While(BrakWhile),
    Return(Option<BrakExpr>),
    Block(BrakBlock),
    Match(BrakMatch),
    Try(BrakTry),
    Throw(BrakExpr),
}

#[derive(Debug, Clone)]
pub struct BrakLet {
    pub name: String,
    pub mutable: bool,
    pub ty: Option<BrakTy>,
    pub value: BrakExpr,
}

#[derive(Debug, Clone)]
pub struct BrakIf {
    pub condition: BrakExpr,
    pub then_block: BrakBlock,
    pub else_block: Option<BrakBlock>,
}

#[derive(Debug, Clone)]
pub struct BrakWhile {
    pub condition: BrakExpr,
    pub body: BrakBlock,
}

#[derive(Debug, Clone)]
pub struct BrakMatch {
    pub expr: Box<BrakExpr>,
    pub arms: Vec<BrakMatchArm>,
}

#[derive(Debug, Clone)]
pub struct BrakMatchArm {
    pub pattern: BrakPattern,
    pub body: BrakExpr,
}

#[derive(Debug, Clone)]
pub enum BrakPattern {
    Wildcard,
    Literal(BrakLiteral),
    Ident(String),
}

#[derive(Debug, Clone)]
pub enum BrakLiteral {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct BrakTry {
    pub try_block: BrakBlock,
    pub catch_var: String,
    pub catch_block: BrakBlock,
}

#[derive(Debug, Clone)]
pub struct BrakHookCall {
    pub kind: BrakHookKind,
}

#[derive(Debug, Clone)]
pub enum BrakHookKind {
    State {
        state_var: String,
        setter_var: String,
        initial: Box<BrakExpr>,
    },
    Effect {
        callback: Box<BrakExpr>,
        deps: Vec<BrakExpr>,
    },
    Memo {
        result_var: String,
        callback: Box<BrakExpr>,
        deps: Vec<BrakExpr>,
    },
}

#[derive(Debug, Clone)]
pub enum BrakExpr {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Ident(String),
    Binary(BrakBinaryOp, Box<BrakExpr>, Box<BrakExpr>),
    Unary(BrakUnaryOp, Box<BrakExpr>),
    Assign(Box<BrakExpr>, Box<BrakExpr>),
    Call(Box<BrakExpr>, Vec<BrakExpr>),
    Member(Box<BrakExpr>, String),
    Index(Box<BrakExpr>, Box<BrakExpr>),
    Array(Vec<BrakExpr>),
    StructInit(String, Vec<(String, BrakExpr)>),
    Block(BrakBlock),
    Ternary(Box<BrakExpr>, Box<BrakExpr>, Box<BrakExpr>),
    ArrowFn(Vec<String>, Box<BrakExpr>),
    Object(Vec<(String, BrakExpr)>),
    Spread(Box<BrakExpr>),
    Template(Vec<BrakExpr>),
    Match(Box<BrakMatch>),
}

#[derive(Debug, Clone, Copy)]
pub enum BrakBinaryOp {
    Add, Sub, Mul, Div, Mod,
    And, Or, Eq, Ne, Lt, Gt, Le, Ge,
    Concat, NullCoalescing,
}

#[derive(Debug, Clone, Copy)]
pub enum BrakUnaryOp {
    Neg, Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrakTy {
    Int(u8),
    UInt(u8),
    Float(u8),
    Bool,
    U8,
    Void,
    Pointer(Box<BrakTy>),
    Array(Box<BrakTy>),
    Optional(Box<BrakTy>),
    Fn(Vec<BrakTy>, Box<BrakTy>),
    Struct(Vec<(String, BrakTy)>),
    Enum(Vec<String>),
    Named(String),
    Any,
}

#[derive(Debug, Clone)]
pub struct BrakProgram {
    pub items: Vec<BrakItem>,
}
