use crate::lexer::TokenKind;
use super::Parser;
use rakit_core::Diagnostic;
use rakit_ir_ast::*;

type Result<T> = std::result::Result<T, Vec<Diagnostic>>;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Min = 0,
    Assign,
    NullCoalescing,
    Ternary,
    Or,
    And,
    Equality,
    Comparison,
    Concat,
    Term,
    Factor,
    Unary,
    Call,
    Member,
}

fn get_precedence(kind: TokenKind) -> Option<Precedence> {
    use Precedence::*;
    match kind {
        TokenKind::Assign => Some(Assign),
        TokenKind::Or => Some(Or),
        TokenKind::And => Some(And),
        TokenKind::Eq | TokenKind::Ne => Some(Equality),
        TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => Some(Comparison),
        TokenKind::Concat => Some(Concat),
        TokenKind::Plus | TokenKind::Minus => Some(Term),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(Factor),
        _ => None,
    }
}

impl<'a> Parser<'a> {
    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_expr_prec(Precedence::Min)
    }

    fn parse_expr_prec(&mut self, min_prec: Precedence) -> Result<Expr> {
        let mut left = self.parse_prefix()?;

        loop {
            // Generic type arguments in function calls: fn<Type>(args) - must be before < binary op
            if self.peek().kind == TokenKind::Lt && self.skippable_generic() {
                self.skip_generic_params();
                continue;
            }

            // Binary operators with precedence
            if let Some(prec) = get_precedence(self.peek().kind) {
                if prec >= min_prec {
                    match self.parse_infix(left, prec) {
                        Ok(new_left) => { left = new_left; continue; }
                        Err(e) => return Err(vec![e]),
                    }
                }
            }

            // Postfix operations: calls, member access, indexing (always tight)
            match self.peek().kind {
                TokenKind::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(&[TokenKind::RParen]) && !self.check(&[TokenKind::Eof]) {
                        args.push(self.parse_expr()?);
                        self.eat(TokenKind::Comma);
                    }
                    self.expect(TokenKind::RParen).map_err(|e| vec![e])?;
                    left = Expr::Call(Box::new(left), args);
                    continue;
                }
                TokenKind::Dot => {
                    self.advance();
                    let name = self.expect_ident_name().map_err(|e| vec![e])?;
                    left = Expr::Member(Box::new(left), name);
                    continue;
                }
                TokenKind::Question if self.peek().lexeme == "?." => {
                    self.advance();
                    let name = self.expect_ident_name().map_err(|e| vec![e])?;
                    left = Expr::Member(Box::new(left), name);
                    continue;
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::RBracket).map_err(|e| vec![e])?;
                    left = Expr::Index(Box::new(left), Box::new(index));
                    continue;
                }
                _ => break,
            }
        }

        if self.eat(TokenKind::Question) {
            if self.check(&[TokenKind::Question]) {
                // ?? nullish coalescing (lowest precedence)
                self.advance();
                let right = self.parse_expr_prec(Precedence::Min)?;
                left = Expr::Binary(BinaryOp::NullCoalescing, Box::new(left), Box::new(right));
            } else {
                let then_expr = self.parse_expr()?;
                if self.eat(TokenKind::Colon) {
                    let else_expr = self.parse_expr()?;
                    left = Expr::Ternary(Box::new(left), Box::new(then_expr), Box::new(else_expr));
                }
            }
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        match self.peek().kind {
            TokenKind::Number | TokenKind::String | TokenKind::CharLit
                | TokenKind::True | TokenKind::False | TokenKind::Null => {
                Ok(self.parse_literal())
            }
            TokenKind::If => {
                // jika (cond) then_expr lain else_expr  — if as expression
                self.advance();
                self.expect(TokenKind::LParen).map_err(|e| vec![e])?;
                let cond = self.parse_expr()?;
                self.expect(TokenKind::RParen).map_err(|e| vec![e])?;
                let then_expr = self.parse_expr()?;
                let else_expr = if self.peek().kind == TokenKind::Else {
                    self.advance();
                    self.parse_expr()?
                } else {
                    Expr::Literal(Literal::Null)
                };
                Ok(Expr::Ternary(Box::new(cond), Box::new(then_expr), Box::new(else_expr)))
            }
            _ if self.peek().is_keyword() => {
                let name = self.advance_lexeme();
                if self.check(&[TokenKind::LBrace]) {
                    self.ensure_loaded(2);
                    let is_object = match self.tokens.get(self.pos + 1).map(|t| t.kind) {
                        Some(TokenKind::RBrace) => true,
                        Some(TokenKind::DotDotDot) => true,
                        Some(TokenKind::Ident) => {
                            match self.tokens.get(self.pos + 2).map(|t| t.kind) {
                                Some(TokenKind::Colon) => true,
                                _ => false,
                            }
                        }
                        _ => false,
                    };
                    if is_object {
                        self.advance();
                        let fields = self.parse_object_fields()?;
                        return Ok(Expr::StructInit(name, fields));
                    }
                }
                Ok(Expr::Ident(name))
            }
            TokenKind::Ident => {
                let name = self.advance_lexeme();
                if self.check(&[TokenKind::LBrace]) {
                    self.ensure_loaded(2);
                    let is_object = match self.tokens.get(self.pos + 1).map(|t| t.kind) {
                        Some(TokenKind::RBrace) => true,
                        Some(TokenKind::DotDotDot) => true,
                        Some(TokenKind::Ident) => {
                            match self.tokens.get(self.pos + 2).map(|t| t.kind) {
                                Some(TokenKind::Colon) => true,
                                _ => false,
                            }
                        }
                        _ => false,
                    };
                    if is_object {
                        self.advance();
                        let fields = self.parse_object_fields()?;
                        return Ok(Expr::StructInit(name, fields));
                    }
                }
                Ok(Expr::Ident(name))
            }
            TokenKind::LParen => {
                self.advance();
                // Check for arrow function: () => or (param) => or (param, param) =>
                if self.check(&[TokenKind::RParen]) {
                    self.advance();
                    if self.check(&[TokenKind::FatArrow]) {
                        return self.parse_arrow_fn_body(Vec::new());
                    }
                    return Ok(Expr::Literal(Literal::Null));
                }
                // Try to parse as arrow function params
                let saved_pos = self.pos;
                let params = self.try_parse_arrow_params();
                if let Some(params) = params {
                    return self.parse_arrow_fn_body(params);
                }
                self.pos = saved_pos;
                // Regular parenthesized expression
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen).map_err(|e| vec![e])?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                // Try anonymous object literal first, fall back to block
                self.ensure_loaded(2);
                let is_object = match self.tokens.get(self.pos + 1).map(|t| t.kind) {
                    Some(TokenKind::RBrace) => true,
                    Some(TokenKind::DotDotDot) => true,
                    Some(TokenKind::Ident) => {
                        match self.tokens.get(self.pos + 2).map(|t| t.kind) {
                            Some(TokenKind::Colon) => true,
                            Some(TokenKind::Comma) => true,
                            Some(TokenKind::RBrace) => true,
                            _ => false,
                        }
                    }
                    Some(TokenKind::String) => true,
                    _ => false,
                };
                if is_object {
                    self.advance();
                    let fields = self.parse_object_fields()?;
                    Ok(Expr::Object(fields))
                } else {
                    let block = self.parse_block().map_err(|e| vec![e])?;
                    Ok(Expr::BlockExpr(block))
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                while !self.check(&[TokenKind::RBracket]) && !self.check(&[TokenKind::Eof]) {
                    if self.eat(TokenKind::DotDotDot) {
                        let expr = self.parse_expr()?;
                        items.push(Expr::Spread(Box::new(expr)));
                    } else {
                        items.push(self.parse_expr()?);
                    }
                    self.eat(TokenKind::Comma);
                }
                self.expect(TokenKind::RBracket).map_err(|e| vec![e])?;
                Ok(Expr::Array(items))
            }
            TokenKind::DotDotDot => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Spread(Box::new(expr)))
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_expr_prec(Precedence::Unary)?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)))
            }
            TokenKind::Bang => {
                self.advance();
                let expr = self.parse_expr_prec(Precedence::Unary)?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)))
            }
            TokenKind::Lt => {
                self.parse_jsx()
            }
            _ => Err(vec![self.error_expected("ekspresi")]),
        }
    }

    fn parse_object_fields(&mut self) -> std::result::Result<Vec<StructInitField>, Vec<Diagnostic>> {
        let mut fields = Vec::new();
        while !self.check(&[TokenKind::RBrace]) && !self.check(&[TokenKind::Eof]) {
            let spread = self.eat(TokenKind::DotDotDot);
            let fname = self.expect_ident().map_err(|e| vec![e])?;
            let fvalue;
            if spread {
                fvalue = Expr::Ident(fname.clone());
            } else if self.check(&[TokenKind::Colon]) {
                self.advance();
                fvalue = self.parse_expr()?;
            } else {
                // Field shorthand: { field } → { field: field }
                fvalue = Expr::Ident(fname.clone());
            }
            fields.push(StructInitField { name: fname, value: fvalue, spread });
            if !self.eat(TokenKind::Comma) { break; }
        }
        self.expect(TokenKind::RBrace).map_err(|e| vec![e])?;
        Ok(fields)
    }

    fn try_parse_arrow_params(&mut self) -> Option<Vec<FnParam>> {
        // Look for pattern: ident or ident: Type, ...
        // After LParen is already consumed
        let saved = self.pos;
        let mut params = Vec::new();
        if self.check(&[TokenKind::RParen]) {
            return Some(Vec::new());
        }
        loop {
            match self.peek().kind {
                TokenKind::Ident | TokenKind::Underscore => {
                    let name = self.advance_lexeme();
                    let ty = if self.eat(TokenKind::Colon) {
                        // Parse type - but we need to save/restore position if this fails
                        self.parse_type().ok().unwrap_or(Type::Infer)
                    } else {
                        Type::Infer
                    };
                    params.push(FnParam {
                        name,
                        ty,
                        span: self.prev_span(),
                    });
                }
                _ => {
                    self.pos = saved;
                    return None;
                }
            }
            if self.eat(TokenKind::Comma) {
                continue;
            }
            break;
        }
        if self.check(&[TokenKind::RParen]) {
            self.advance(); // consume )
            if self.check(&[TokenKind::FatArrow]) {
                return Some(params);
            }
        }
        self.pos = saved;
        None
    }

    fn parse_arrow_fn_body(&mut self, params: Vec<FnParam>) -> Result<Expr> {
        self.advance(); // consume =>
        let body = self.parse_expr()?;
        Ok(Expr::ArrowFn(ArrowFn {
            params,
            body: Box::new(body),
            span: self.prev_span(),
        }))
    }

    fn parse_infix(&mut self, left: Expr, _min_prec: Precedence) -> std::result::Result<Expr, Diagnostic> {
        let kind = self.peek().kind;

        match kind {
            TokenKind::Assign => {
                self.advance();
                let value = self.parse_expr_prec(Precedence::Assign).map_err(|mut v| v.remove(0))?;
                Ok(Expr::Assign {
                    target: Box::new(left),
                    value: Box::new(value),
                })
            }
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash
                | TokenKind::Percent | TokenKind::Eq | TokenKind::Ne
                | TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge
                | TokenKind::And | TokenKind::Or | TokenKind::Concat => {
                let op = Self::token_to_binary_op(kind);
                self.advance();
                let prec = get_precedence(kind).unwrap();
                let right = self.parse_expr_prec(prec).map_err(|mut v| v.remove(0))?;
                Ok(Expr::Binary(op, Box::new(left), Box::new(right)))
            }
            _ => Err(self.error_expected("operator")),
        }
    }

    // ── Literals ──────────────────────────────────────────────────

    fn parse_literal(&mut self) -> Expr {
        let tok = self.peek().clone();
        self.advance();
        match tok.kind {
            TokenKind::Number => {
                let val: f64 = tok.lexeme.parse().unwrap_or(0.0);
                Expr::Literal(Literal::Number(val))
            }
            TokenKind::String => {
                let s = Self::unescape_string(&tok.lexeme);
                Expr::Literal(Literal::String(s))
            }
            TokenKind::CharLit => {
                let c = tok.lexeme.chars().nth(1).unwrap_or('\0');
                Expr::Literal(Literal::Char(c))
            }
            TokenKind::True => Expr::Literal(Literal::Bool(true)),
            TokenKind::False => Expr::Literal(Literal::Bool(false)),
            TokenKind::Null => Expr::Literal(Literal::Null),
            _ => unreachable!(),
        }
    }

    fn unescape_string(s: &str) -> String {
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
                        Some('r') => out.push('\r'),
                        Some('\\') => out.push('\\'),
                        Some('"') => out.push('"'),
                        Some('\'') => out.push('\''),
                        Some(c) => { out.push('\\'); out.push(c); }
                        None => break,
                    }
                }
                Some(c) => out.push(c),
            }
        }
        out
    }

    // ── JSX ───────────────────────────────────────────────────────

    fn parse_jsx(&mut self) -> Result<Expr> {
        self.advance();

        if self.eat(TokenKind::Gt) {
            return self.parse_jsx_fragment();
        }

        if self.eat(TokenKind::Slash) {
            return Err(vec![self.error_expected("tag JSX")]);
        }

        let mut tag_parts = vec![self.expect_ident_name().map_err(|e| vec![e])?];
        while self.eat(TokenKind::Dot) {
            tag_parts.push(self.expect_ident_name().map_err(|e| vec![e])?);
        }
        let tag = tag_parts.join(".");
        let mut attrs = Vec::new();

        loop {
            match self.peek().kind {
                kind if kind.is_ident_like() => {
                    let attr_name = self.advance_lexeme();
                    if self.eat(TokenKind::Assign) {
                        if self.eat(TokenKind::LBrace) {
                            let value = self.parse_expr()?;
                            self.expect(TokenKind::RBrace).map_err(|e| vec![e])?;
                            attrs.push(JsxAttr::Expr {
                                name: attr_name,
                                value,
                                span: self.prev_span(),
                            });
                        } else {
                            let tok = self.peek().clone();
                            if tok.kind == TokenKind::String {
                                self.advance();
                                let val = Self::unescape_string(&tok.lexeme);
                                attrs.push(JsxAttr::Literal {
                                    name: attr_name,
                                    value: val,
                                    span: tok.span,
                                });
                            } else {
                                return Err(vec![self.error_expected("nilai atribut JSX")]);
                            }
                        }
                    } else {
                        attrs.push(JsxAttr::Literal {
                            name: attr_name,
                            value: "true".to_string(),
                            span: self.prev_span(),
                        });
                    }
                }
                TokenKind::LBrace => {
                    self.advance();
                    attrs.push(JsxAttr::Spread(
                        self.parse_expr()?,
                        self.prev_span(),
                    ));
                    self.expect(TokenKind::RBrace).map_err(|e| vec![e])?;
                }
                TokenKind::Slash => {
                    self.advance();
                    self.expect(TokenKind::Gt).map_err(|e| vec![e])?;
                    return Ok(Expr::JsxElement(Box::new(JsxElement {
                        tag,
                        attrs,
                        children: Vec::new(),
                        span: self.prev_span(),
                    })));
                }
                TokenKind::Gt => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(vec![self.error_expected("atribut atau > JSX")])
                },
            }
        }

        let mut children = Vec::new();
        loop {
            match self.peek().kind {
                TokenKind::Lt => {
                    let saved_pos = self.pos;
                    self.advance();
                    if self.eat(TokenKind::Slash) {
                        let mut close_parts = vec![self.expect_ident_name().map_err(|e| vec![e])?];
                        while self.eat(TokenKind::Dot) {
                            close_parts.push(self.expect_ident_name().map_err(|e| vec![e])?);
                        }
                        let close_tag = close_parts.join(".");
                        self.expect(TokenKind::Gt).map_err(|e| vec![e])?;
                        if close_tag != tag {
                            return Err(vec![self.error_expected(&format!("</{}>", tag))]);
                        }
                        break;
                    } else {
                        self.pos = saved_pos;
                        let child = self.parse_jsx()?;
                        match child {
                            Expr::JsxElement(e) => children.push(JsxChild::Element(e)),
                            Expr::JsxFragment(f) => children.push(JsxChild::Fragment(f)),
                            _ => {}
                        }
                    }
                }
                TokenKind::LBrace => {
                    self.advance();
                    let expr = self.parse_expr()?;
                    self.expect(TokenKind::RBrace).map_err(|e| vec![e])?;
                    children.push(JsxChild::Expr(expr));
                }
                TokenKind::String => {
                    let tok = self.peek().clone();
                    self.advance();
                    children.push(JsxChild::Text(Self::unescape_string(&tok.lexeme)));
                }
                _ => {
                    let text = self.collect_jsx_text();
                    if !text.is_empty() {
                        children.push(JsxChild::Text(text));
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(Expr::JsxElement(Box::new(JsxElement {
            tag,
            attrs,
            children,
            span: self.prev_span(),
        })))
    }

    fn parse_jsx_fragment(&mut self) -> Result<Expr> {
        let mut children = Vec::new();

        loop {
            match self.peek().kind {
                TokenKind::Lt => {
                    let saved_pos = self.pos;
                    self.advance();
                    if self.eat(TokenKind::Slash) {
                        self.expect(TokenKind::Gt).map_err(|e| vec![e])?;
                        break;
                    } else {
                        self.pos = saved_pos;
                        let child = self.parse_jsx()?;
                        match child {
                            Expr::JsxElement(e) => children.push(JsxChild::Element(e)),
                            Expr::JsxFragment(f) => children.push(JsxChild::Fragment(f)),
                            _ => {}
                        }
                    }
                }
                TokenKind::LBrace => {
                    self.advance();
                    let expr = self.parse_expr()?;
                    self.expect(TokenKind::RBrace).map_err(|e| vec![e])?;
                    children.push(JsxChild::Expr(expr));
                }
                _ => {
                    let text = self.collect_jsx_text();
                    if !text.is_empty() {
                        children.push(JsxChild::Text(text));
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(Expr::JsxFragment(Box::new(JsxFragment {
            children,
            span: self.prev_span(),
        })))
    }

    fn collect_jsx_text(&mut self) -> String {
        let mut text = String::new();
        while self.pos < self.tokens.len() {
            match self.peek().kind {
                TokenKind::Lt | TokenKind::LBrace | TokenKind::Eof => break,
                _ => {
                    text.push_str(&self.peek().lexeme);
                    self.advance();
                }
            }
        }
        text
    }

    // ── Helpers ───────────────────────────────────────────────────

    fn token_to_binary_op(kind: TokenKind) -> BinaryOp {
        use TokenKind::*;
        match kind {
            Plus => BinaryOp::Add,
            Minus => BinaryOp::Sub,
            Star => BinaryOp::Mul,
            Slash => BinaryOp::Div,
            Percent => BinaryOp::Mod,
            Eq => BinaryOp::Eq,
            Ne => BinaryOp::Ne,
            Lt => BinaryOp::Lt,
            Gt => BinaryOp::Gt,
            Le => BinaryOp::Le,
            Ge => BinaryOp::Ge,
            And => BinaryOp::And,
            Or => BinaryOp::Or,
            Concat => BinaryOp::Concat,
            _ => unreachable!(),
        }
    }
}
