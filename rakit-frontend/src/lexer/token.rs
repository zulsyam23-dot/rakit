use rakit_core::Span;
use super::token_kind::TokenKind;

/// Sebuah token hasil lexing.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, span: Span) -> Self {
        Token {
            kind,
            lexeme: lexeme.into(),
            span,
        }
    }

    pub fn is(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    pub fn is_keyword(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Fn
                | TokenKind::Component
                | TokenKind::Let
                | TokenKind::Mut
                | TokenKind::Type
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::While
                | TokenKind::For
                | TokenKind::In
                | TokenKind::Match
                | TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Import
                | TokenKind::From
                | TokenKind::Export
                | TokenKind::Try
                | TokenKind::Catch
                | TokenKind::Throw
                | TokenKind::Render
                | TokenKind::State
                | TokenKind::Effect
                | TokenKind::Ref
                | TokenKind::Context
                | TokenKind::As
                | TokenKind::Wildcard
                | TokenKind::MutKw
        )
    }
}
