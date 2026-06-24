use crate::lexer::TokenKind;
use super::Parser;
use rakit_core::Diagnostic;
use rakit_ir_ast::*;

type Result<T> = std::result::Result<T, Vec<Diagnostic>>;

impl<'a> Parser<'a> {
    /// Parse satu statement.
    pub fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek().kind {
            TokenKind::Let => self.parse_let(false),
            TokenKind::Mut => self.parse_let(true),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Match => self.parse_match(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break => {
                self.advance();
                self.eat(TokenKind::Semicolon);
                Ok(Stmt::Break(self.prev_span()))
            }
            TokenKind::Continue => {
                self.advance();
                self.eat(TokenKind::Semicolon);
                Ok(Stmt::Continue(self.prev_span()))
            }
            TokenKind::Try => self.parse_try(),
            TokenKind::Throw => self.parse_throw(),
            TokenKind::LBrace => {
                let block = self.parse_block().map_err(|e| vec![e])?;
                Ok(Stmt::Block(block))
            }
            _ => {
                let expr = self.parse_expr()?;
                let span = self.current_span();
                self.eat(TokenKind::Semicolon);
                Ok(Stmt::Expr(expr, span))
            }
        }
    }

    fn parse_let(&mut self, mutable: bool) -> Result<Stmt> {
        let start = self.current_span();
        let _keyword = self.advance_lexeme();

        let name = self.expect_ident().map_err(|e| vec![e])?;

        let ty = if self.eat(TokenKind::Colon) {
            Some(self.parse_type().map_err(|e| vec![e])?)
        } else {
            None
        };

        let value = if self.eat(TokenKind::Assign) {
            self.parse_expr()?
        } else {
            Expr::Literal(Literal::Null)
        };

        self.eat(TokenKind::Semicolon);

        Ok(Stmt::Let(LetDef {
            name,
            mutable,
            ty,
            value,
            span: start,
        }))
    }

    fn parse_if(&mut self) -> Result<Stmt> {
        let start = self.current_span();
        self.advance();

        let condition = if self.check(&[TokenKind::LBrace]) {
            Expr::Literal(Literal::Bool(true))
        } else {
            self.parse_expr()?
        };

        let then_block = self.parse_block().map_err(|e| vec![e])?;

        let else_block = if self.peek().kind == TokenKind::Else {
            self.advance();
            if self.peek().kind == TokenKind::If {
                let inner = self.parse_if()?;
                Some(Block {
                    stmts: vec![inner],
                    span: self.prev_span(),
                })
            } else {
                Some(self.parse_block().map_err(|e| vec![e])?)
            }
        } else {
            None
        };

        Ok(Stmt::If(IfStmt {
            condition,
            then_block,
            else_block,
            span: start,
        }))
    }

    fn parse_while(&mut self) -> Result<Stmt> {
        let start = self.current_span();
        self.advance();

        let condition = if self.check(&[TokenKind::LBrace]) {
            Expr::Literal(Literal::Bool(true))
        } else {
            self.parse_expr()?
        };

        let body = self.parse_block().map_err(|e| vec![e])?;

        Ok(Stmt::While(WhileStmt {
            condition,
            body,
            span: start,
            label: None,
        }))
    }

    fn parse_for(&mut self) -> Result<Stmt> {
        let start = self.current_span();
        self.advance();

        let _var = self.expect_ident().map_err(|e| vec![e])?;
        self.expect(TokenKind::In).map_err(|e| vec![e])?;
        let _iterable = self.parse_expr()?;
        let body = self.parse_block().map_err(|e| vec![e])?;

        Ok(Stmt::While(WhileStmt {
            condition: Expr::Literal(Literal::Bool(true)),
            body,
            span: start,
            label: None,
        }))
    }

    fn parse_match(&mut self) -> Result<Stmt> {
        let start = self.current_span();
        self.advance();

        let expr = self.parse_expr()?;
        self.expect(TokenKind::LBrace).map_err(|e| vec![e])?;

        let mut arms = Vec::new();
        while !self.check(&[TokenKind::RBrace]) && !self.check(&[TokenKind::Eof]) {
            arms.push(self.parse_match_arm()?);
            self.eat(TokenKind::Comma);
        }
        self.expect(TokenKind::RBrace).map_err(|e| vec![e])?;

        Ok(Stmt::Match(MatchStmt {
            expr,
            arms,
            span: start,
        }))
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let pattern = self.parse_pattern()?;
        self.expect(TokenKind::FatArrow).map_err(|e| vec![e])?;
        let body = self.parse_expr()?;
        let span = self.prev_span();
        Ok(MatchArm {
            pattern,
            body,
            span,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match self.peek().kind {
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::Number | TokenKind::String | TokenKind::True
                | TokenKind::False | TokenKind::Null => {
                let tok = self.peek().clone();
                self.advance();
                let lit = match tok.kind {
                    TokenKind::Number => Literal::Number(tok.lexeme.parse().unwrap_or(0.0)),
                    TokenKind::String => Literal::String(self.unescape_pattern_string(&tok.lexeme)),
                    TokenKind::True => Literal::Bool(true),
                    TokenKind::False => Literal::Bool(false),
                    TokenKind::Null => Literal::Null,
                    _ => unreachable!(),
                };
                Ok(Pattern::Literal(lit))
            }
            TokenKind::Ident => {
                let name = self.advance_lexeme();
                Ok(Pattern::Ident(name))
            }
            _ => Err(vec![self.error_expected("pola pattern")]),
        }
    }

    fn parse_return(&mut self) -> Result<Stmt> {
        let start = self.current_span();
        self.advance();

        if self.check(&[TokenKind::Semicolon, TokenKind::RBrace]) {
            self.eat(TokenKind::Semicolon);
            Ok(Stmt::Return(None, start))
        } else {
            let expr = self.parse_expr()?;
            self.eat(TokenKind::Semicolon);
            Ok(Stmt::Return(Some(expr), start))
        }
    }

    fn parse_try(&mut self) -> Result<Stmt> {
        let start = self.current_span();
        self.advance();

        let try_block = self.parse_block().map_err(|e| vec![e])?;

        self.expect(TokenKind::Catch).map_err(|e| vec![e])?;
        let catch_var = self.expect_ident().map_err(|e| vec![e])?;
        let catch_block = self.parse_block().map_err(|e| vec![e])?;

        Ok(Stmt::Try(TryStmt {
            try_block,
            catch_var,
            catch_block,
            span: start,
        }))
    }

    fn parse_throw(&mut self) -> Result<Stmt> {
        let start = self.current_span();
        self.advance();

        let expr = self.parse_expr()?;
        self.eat(TokenKind::Semicolon);

        Ok(Stmt::Throw(expr, start))
    }

    fn unescape_pattern_string(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        chars.next();
        loop {
            match chars.next() {
                None | Some('"') => break,
                Some('\\') => {
                    match chars.next() {
                        Some('n') => out.push('\n'),
                        Some('t') => out.push('\t'),
                        Some('"') => out.push('"'),
                        Some(c) => out.push(c),
                        None => break,
                    }
                }
                Some(c) => out.push(c),
            }
        }
        out
    }
}
