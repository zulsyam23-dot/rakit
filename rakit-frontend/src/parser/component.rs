use crate::lexer::TokenKind;
use super::Parser;
use rakit_core::Result;
use rakit_ir_ast::*;

/// Parser untuk JSX elements dan komponen Rakit.
/// Parser utama untuk JSX ada di expr.rs (parse_jsx, parse_jsx_fragment).
/// Module ini menyediakan helper tambahan untuk parsing dalam konteks komponen.
impl<'a> Parser<'a> {
    /// Parse dekorator / atribut komponen (jika ada).
    /// Contoh: @efek, @keadaan, @ref
    pub fn parse_component_decorator(&mut self) -> Result<Option<String>> {
        if self.eat(TokenKind::At) {
            let name = self.expect_ident()?;
            Ok(Some(name))
        } else {
            Ok(None)
        }
    }

    /// Parse ekspresi JSX dalam konteks komponen.
    pub fn parse_jsx_in_component(&mut self) -> Result<Expr> {
        // Cek apakah kita di dalam < atau <>
        match self.peek().kind {
            TokenKind::Lt => self.parse_jsx(),
            _ => self.parse_expr(),
        }
    }

    /// Parse hook calls: keadaan, efek, ref
    pub fn parse_hook_call(&mut self) -> Result<Expr> {
        match self.peek().kind {
            TokenKind::State => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let initial = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Call(
                    Box::new(Expr::Ident("keadaan".to_string())),
                    vec![initial],
                ))
            }
            TokenKind::Effect => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let callback = self.parse_expr()?;
                let mut deps = Vec::new();
                if self.eat(TokenKind::Comma) {
                    // Parse array of deps
                    deps.push(self.parse_expr()?);
                }
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Call(
                    Box::new(Expr::Ident("efek".to_string())),
                    vec![callback],
                ))
            }
            TokenKind::Ref => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let initial = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Call(
                    Box::new(Expr::Ident("ref".to_string())),
                    vec![initial],
                ))
            }
            _ => self.parse_expr(),
        }
    }
}
