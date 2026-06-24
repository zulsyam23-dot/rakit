pub mod expr;
pub mod stmt;
// pub mod component;

use crate::lexer::{Lexer, Token, TokenKind};
use rakit_core::{Span, SourceMap, Diagnostic};
use rakit_ir_ast::*;

type Result<T> = std::result::Result<T, Vec<Diagnostic>>;

/// Parser recursive descent untuk Rakit.
pub struct Parser<'a> {
    _source: &'a str,
    lexer: Lexer<'a>,
    tokens: Vec<Token>,
    pos: usize,
    source_map: &'a SourceMap,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, source_map: &'a SourceMap) -> Self {
        let lexer = Lexer::new(source, source_map);
        let mut parser = Parser {
            _source: source,
            lexer,
            tokens: Vec::new(),
            pos: 0,
            source_map,
            diagnostics: Vec::new(),
        };
        parser.load_next(); // load first token into tokens[0]
        parser
    }

    /// Entry point: parse seluruh program.
    pub fn parse_program(mut self) -> Result<Program> {
        let mut items = Vec::new();
        let start = self.current_span();
        while !self.check(&[TokenKind::Eof]) {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(diag) => {
                    self.diagnostics.push(diag);
                    self.sync_to_next_item();
                }
            }
        }
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }
        let end = self.current_span();
        Ok(Program {
            items,
            span: start.merge(end),
        })
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    fn parse_item(&mut self) -> std::result::Result<Item, Diagnostic> {
        match self.peek().kind {
            TokenKind::Fn => self.parse_fn_item(),
            TokenKind::Component => self.parse_component_item(),
            TokenKind::Struct => self.parse_struct_item(),
            TokenKind::Enum => self.parse_enum_item(),
            TokenKind::Type => self.parse_type_alias(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Export => self.parse_export(),
            _ => Err(self.error_expected("item (fungsi/komponen/struk/pilihan/tipe/impor/ekspor)")),
        }
    }

    fn parse_fn_item(&mut self) -> std::result::Result<Item, Diagnostic> {
        let start = self.current_span();
        self.advance();

        let name = self.expect_ident()?;

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(&[TokenKind::RParen]) {
            loop {
                params.push(self.parse_fn_param()?);
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;

        let return_ty = if self.eat(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let end = self.current_span();

        Ok(Item::Function(FnDef {
            name,
            params,
            return_ty,
            body,
            span: start.merge(end),
        }))
    }

    fn parse_component_item(&mut self) -> std::result::Result<Item, Diagnostic> {
        let start = self.current_span();
        self.advance();

        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;

        let mut props = Vec::new();
        if !self.check(&[TokenKind::RParen]) {
            loop {
                props.push(self.parse_fn_param()?);
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;

        let body = self.parse_component_body()?;
        let end = self.current_span();

        Ok(Item::Component(ComponentDef {
            name,
            props,
            body,
            span: start.merge(end),
        }))
    }

    fn parse_struct_item(&mut self) -> std::result::Result<Item, Diagnostic> {
        let start = self.current_span();
        self.advance();

        let name = self.expect_ident()?;

        let generics = if self.eat(TokenKind::Lt) {
            let mut g = Vec::new();
            loop {
                g.push(self.expect_ident()?);
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::Gt)?;
            g
        } else {
            Vec::new()
        };

        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&[TokenKind::RBrace]) && !self.check(&[TokenKind::Eof]) {
            let field_name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let field_ty = self.parse_type()?;
            let field_span = self.prev_span();
            fields.push(StructField {
                name: field_name,
                ty: field_ty,
                span: field_span,
            });
            self.eat(TokenKind::Comma);
        }
        self.expect(TokenKind::RBrace)?;
        let end = self.current_span();

        Ok(Item::Struct(StructDef {
            name,
            fields,
            generics,
            span: start.merge(end),
        }))
    }

    fn parse_enum_item(&mut self) -> std::result::Result<Item, Diagnostic> {
        let start = self.current_span();
        self.advance();

        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while !self.check(&[TokenKind::RBrace]) && !self.check(&[TokenKind::Eof]) {
            let var_name = self.expect_ident()?;
            let var_span = self.prev_span();
            let mut fields = Vec::new();
            if self.eat(TokenKind::LParen) {
                loop {
                    fields.push(self.parse_type()?);
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
            }
            variants.push(EnumVariant {
                name: var_name,
                fields,
                span: var_span,
            });
            self.eat(TokenKind::Comma);
        }
        self.expect(TokenKind::RBrace)?;
        let end = self.current_span();

        Ok(Item::Enum(EnumDef {
            name,
            variants,
            span: start.merge(end),
        }))
    }

    fn parse_type_alias(&mut self) -> std::result::Result<Item, Diagnostic> {
        let start = self.current_span();
        self.advance();

        let name = self.expect_ident()?;
        self.expect(TokenKind::Assign)?;
        let ty = self.parse_type()?;
        self.eat(TokenKind::Semicolon);
        let end = self.current_span();

        Ok(Item::TypeAlias(TypeAlias {
            name,
            ty,
            span: start.merge(end),
        }))
    }

    fn parse_import(&mut self) -> std::result::Result<Item, Diagnostic> {
        let start = self.current_span();
        self.advance();

        self.expect(TokenKind::String)?;
        let module = self.prev_lexeme();

        self.expect(TokenKind::LBrace)?;
        let mut names = Vec::new();
        if !self.check(&[TokenKind::RBrace]) {
            loop {
                names.push(self.expect_ident()?);
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RBrace)?;
        self.eat(TokenKind::Semicolon);
        let end = self.current_span();

        Ok(Item::Import(Import {
            module,
            names,
            span: start.merge(end),
        }))
    }

    fn parse_export(&mut self) -> std::result::Result<Item, Diagnostic> {
        let start = self.current_span();
        self.advance();

        let item_name = if self.check(&[TokenKind::Fn, TokenKind::Component, TokenKind::Struct, TokenKind::Enum, TokenKind::Type]) {
            self.advance();
            self.expect_ident()?
        } else {
            self.expect_ident()?
        };
        let end = self.current_span();

        Ok(Item::Export(Export {
            item_name,
            span: start.merge(end),
        }))
    }

    // ── Param / Type / Block / Component Body ─────────────────────

    fn parse_fn_param(&mut self) -> std::result::Result<FnParam, Diagnostic> {
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Ok(FnParam {
            name,
            ty,
            span: self.prev_span(),
        })
    }

    pub fn parse_type(&mut self) -> std::result::Result<Type, Diagnostic> {
        match self.peek().kind {
            TokenKind::Ident => {
                let name = self.advance_lexeme();
                if self.eat(TokenKind::Lt) {
                    let mut params = Vec::new();
                    loop {
                        params.push(self.parse_type()?);
                        if !self.eat(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::Gt)?;
                    Ok(Type::Generic(rakit_ir_ast::GenericType {
                        name,
                        params,
                        span: self.prev_span(),
                    }))
                } else if self.eat(TokenKind::Question) {
                    Ok(Type::Optional(Box::new(Type::Named(name))))
                } else {
                    Ok(Type::Named(name))
                }
            }
            TokenKind::LParen => {
                self.advance();
                let mut params = Vec::new();
                loop {
                    params.push(self.parse_type()?);
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                if self.eat(TokenKind::Arrow) {
                    let ret = self.parse_type()?;
                    Ok(Type::Fn(params, Box::new(ret)))
                } else {
                    Ok(Type::Tuple(params))
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                Ok(Type::Array(Box::new(inner)))
            }
            TokenKind::Underscore => {
                self.advance();
                Ok(Type::Infer)
            }
            _ => Err(self.error_expected("tipe")),
        }
    }

    pub fn parse_block(&mut self) -> std::result::Result<Block, Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&[TokenKind::RBrace]) && !self.check(&[TokenKind::Eof]) {
            stmts.push(self.parse_stmt().map_err(|mut v| v.remove(0))?);
        }
        self.expect(TokenKind::RBrace)?;
        let end = self.current_span();
        Ok(Block {
            stmts,
            span: start.merge(end),
        })
    }

    fn parse_component_body(&mut self) -> std::result::Result<ComponentBody, Diagnostic> {
        self.expect(TokenKind::LBrace)?;
        let mut statements = Vec::new();
        let mut render = None;

        while !self.check(&[TokenKind::RBrace]) && !self.check(&[TokenKind::Eof]) {
            if self.check_keyword(TokenKind::Render) {
                self.advance();
                render = Some(self.parse_expr().map_err(|mut v| v.remove(0))?);
                self.eat(TokenKind::Semicolon);
            } else {
                statements.push(self.parse_stmt().map_err(|mut v| v.remove(0))?);
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(ComponentBody {
            statements,
            render: render.unwrap_or(Expr::Literal(Literal::Null)),
        })
    }

    // ── Parser state helpers ──────────────────────────────────────

    fn load_next(&mut self) {
        match self.lexer.next() {
            Some(Ok(tok)) => self.tokens.push(tok),
            Some(Err(diag)) => {
                self.diagnostics.push(diag);
                self.tokens.push(Token::new(TokenKind::Error, "", Span::empty(self.source_map.id())));
            }
            None => {
                self.tokens.push(Token::new(TokenKind::Eof, "", Span::empty(self.source_map.id())));
            }
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
        if self.pos >= self.tokens.len() {
            self.load_next();
        }
    }

    fn advance_lexeme(&mut self) -> String {
        let lexeme = self.peek().lexeme.clone();
        self.advance();
        lexeme
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn ensure_loaded(&mut self, count: usize) {
        while self.tokens.len() <= self.pos + count {
            self.load_next();
        }
    }

    fn check(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.peek().kind)
    }

    fn check_keyword(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> std::result::Result<(), Diagnostic> {
        if self.peek().kind == kind {
            self.advance();
            Ok(())
        } else {
            Err(self.error_expected(&format!("{:?}", kind)))
        }
    }

    fn expect_ident(&mut self) -> std::result::Result<String, Diagnostic> {
        if self.peek().kind == TokenKind::Ident {
            Ok(self.advance_lexeme())
        } else {
            Err(self.error_expected("identifier"))
        }
    }

    fn current_span(&self) -> Span {
        self.peek().span
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::empty(self.source_map.id())
        }
    }

    fn prev_lexeme(&self) -> String {
        if self.pos > 0 {
            self.tokens[self.pos - 1].lexeme.clone()
        } else {
            String::new()
        }
    }

    fn error_expected(&self, expected: &str) -> Diagnostic {
        Diagnostic::error(format!(
            "Diharapkan {}, ditemukan {:?} ('{}')",
            expected,
            self.peek().kind,
            self.peek().lexeme
        ))
        .at(self.peek().span)
    }

    fn sync_to_next_item(&mut self) {
        loop {
            let kind = self.peek().kind;
            if kind == TokenKind::Eof {
                return;
            }
            if matches!(
                kind,
                TokenKind::Fn | TokenKind::Component | TokenKind::Struct
                    | TokenKind::Enum | TokenKind::Type | TokenKind::Import
                    | TokenKind::Export
            ) {
                return;
            }
            self.advance();
        }
    }
}
