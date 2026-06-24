/// Seluruh jenis token yang dikenal oleh lexer Rakit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // ── Keyword Bahasa Indonesia ──────────────────────────────────
    Fn,          // "fungsi"
    Component,   // "komponen"
    Let,         // "konstan"
    Mut,         // "ubah"
    Type,        // "tipe"
    Struct,      // "struk"
    Enum,        // "pilihan"
    If,          // "jika"
    Else,        // "lain"
    While,       // "ulang"
    For,         // "untuk"
    In,          // "dalam"
    Match,       // "cocok"
    Return,      // "berhenti"
    Break,       // "berhenti" (contextual — same keyword, different token)
    Continue,    // "lanjut"
    True,        // "benar"
    False,       // "salah"
    Null,        // "batal"
    Import,      // "impor"
    From,        // "dari"
    Export,      // "ekspor"
    Try,         // "coba"
    Catch,       // "tangkap"
    Throw,       // "lempar"
    Render,      // "tampilkan"
    State,       // "keadaan"
    Effect,      // "efek"
    Ref,         // "ref"
    Context,     // "konteks"
    As,          // "sebagai"
    Wildcard,    // "semua"
    MutKw,       // "ubah"

    // ── Punctuation ───────────────────────────────────────────────
    LParen,       // (
    RParen,       // )
    LBrace,       // {
    RBrace,       // }
    LBracket,     // [
    RBracket,     // ]
    Comma,        // ,
    Semicolon,    // ;
    Colon,        // :
    Dot,          // .
    Arrow,        // ->
    FatArrow,     // =>
    Question,     // ?
    At,           // @
    Hash,         // #
    Underscore,   // _
    Pipe,         // |

    // ── Operators ─────────────────────────────────────────────────
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Eq,           // ==
    Ne,           // !=
    Lt,           // <
    Gt,           // >
    Le,           // <=
    Ge,           // >=
    Assign,       // =
    And,          // &&
    Or,           // ||
    Bang,         // !
    Concat,       // ++

    // ── JSX Tokens ────────────────────────────────────────────────
    JsxOpen,      // < (saat identifier pertama setelah < adalah tag)
    JsxClose,     // </
    JsxSelfClose, // />
    JsxExprOpen,  // { (JSX expression mode)
    JsxExprClose, // } (JSX expression mode)

    // ── Range ─────────────────────────────────────────────────────
    DotDot,       // ..
    DotDotDot,    // ...

    // ── Literals ──────────────────────────────────────────────────
    Number,       // 123, 3.14
    String,       // "halo"
    CharLit,      // 'a'
    Ident,        // nama_variabel

    // ── Special ───────────────────────────────────────────────────
    Comment,      // // line atau /* block */
    Error,        // token tak dikenal
    Eof,          // akhir file
}
