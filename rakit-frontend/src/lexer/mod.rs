mod token_kind;
mod token;

pub use token_kind::TokenKind;
pub use token::Token;

use rakit_core::{Span, SourceMap, Diagnostic};
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Diagnostic>;

/// Iterator over chars with byte-position tracking.
struct CharStream<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pos: usize,       // byte offset of current character
    current: Option<char>, // current character (None if EOF)
}

impl<'a> CharStream<'a> {
    fn new(source: &'a str) -> Self {
        let mut chars = source.char_indices().peekable();
        let (pos, current) = match chars.next() {
            Some((i, c)) => (i, Some(c)),
            None => (0, None),
        };
        CharStream { source, chars, pos, current }
    }

    fn peek(&mut self, offset: usize) -> Option<char> {
        if offset == 0 {
            return self.current;
        }
        let mut iter = self.chars.clone();
        let mut result = None;
        for _ in 0..offset {
            result = iter.next().map(|(_, c)| c);
        }
        result
    }

    fn advance(&mut self) {
        if let Some((i, c)) = self.chars.next() {
            self.pos = i;
            self.current = Some(c);
        } else {
            self.pos = self.source.len();
            self.current = None;
        }
    }

    fn is_eof(&self) -> bool {
        self.current.is_none()
    }
}

/// Lexer untuk bahasa Rakit.
pub struct Lexer<'a> {
    stream: CharStream<'a>,
    source: &'a str,
    start: usize,       // byte offset of current token start
    start_pos: usize,   // char index for error reporting
    source_map: &'a SourceMap,
    keywords: HashMap<&'static str, TokenKind>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, source_map: &'a SourceMap) -> Self {
        let keywords = Self::build_keywords();
        Lexer {
            stream: CharStream::new(source),
            source,
            start: 0,
            start_pos: 0,
            source_map,
            keywords,
        }
    }

    fn build_keywords() -> HashMap<&'static str, TokenKind> {
        let mut m = HashMap::new();
        m.insert("fungsi",    TokenKind::Fn);
        m.insert("komponen",  TokenKind::Component);
        m.insert("konstan",   TokenKind::Let);
        m.insert("ubah",      TokenKind::Mut);
        m.insert("tipe",      TokenKind::Type);
        m.insert("struk",     TokenKind::Struct);
        m.insert("pilihan",   TokenKind::Enum);
        m.insert("jika",      TokenKind::If);
        m.insert("lain",      TokenKind::Else);
        m.insert("ulang",     TokenKind::While);
        m.insert("untuk",     TokenKind::For);
        m.insert("dalam",     TokenKind::In);
        m.insert("cocok",     TokenKind::Match);
        m.insert("berhenti",  TokenKind::Return);
        m.insert("lanjut",    TokenKind::Continue);
        m.insert("benar",     TokenKind::True);
        m.insert("salah",     TokenKind::False);
        m.insert("batal",     TokenKind::Null);
        m.insert("impor",     TokenKind::Import);
        m.insert("dari",      TokenKind::From);
        m.insert("ekspor",    TokenKind::Export);
        m.insert("coba",      TokenKind::Try);
        m.insert("tangkap",   TokenKind::Catch);
        m.insert("lempar",    TokenKind::Throw);
        m.insert("tampilkan", TokenKind::Render);
        m.insert("keadaan",   TokenKind::State);
        m.insert("efek",      TokenKind::Effect);
        m.insert("ref",       TokenKind::Ref);
        m.insert("konteks",   TokenKind::Context);
        m.insert("sebagai",   TokenKind::As);
        m.insert("semua",     TokenKind::Wildcard);
        m.insert("jalan",     TokenKind::Try);
        m
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        self.start = self.stream.pos;
        self.start_pos = self.stream.pos;

        if self.stream.is_eof() {
            return Ok(self.make_token(TokenKind::Eof));
        }

        let c = self.stream.current.unwrap();

        if c.is_alphabetic() || c == '_' {
            return self.read_identifier();
        }

        if c.is_ascii_digit() {
            return self.read_number();
        }

        match c {
            '"' => self.read_string(),
            '\'' => self.read_char(),
            '/' if self.stream.peek(1) == Some('/') => { self.skip_comment_line(); self.next_token() }
            '/' if self.stream.peek(1) == Some('*') => { self.skip_comment_block(); self.next_token() }
            _ => self.read_operator(),
        }
    }

    fn read_identifier(&mut self) -> Result<Token> {
        self.stream.advance();
        while let Some(c) = self.stream.current {
            if c.is_alphanumeric() || c == '_' {
                self.stream.advance();
            } else {
                break;
            }
        }
        let lexeme = &self.source[self.start..self.stream.pos];
        let kind = self.keywords.get(lexeme).copied().unwrap_or(TokenKind::Ident);
        Ok(self.make_token(kind))
    }

    fn read_number(&mut self) -> Result<Token> {
        self.stream.advance();
        while let Some(c) = self.stream.current {
            if c.is_ascii_digit() {
                self.stream.advance();
            } else {
                break;
            }
        }
        if self.stream.current == Some('.') && self.stream.peek(1).map_or(false, |c| c.is_ascii_digit()) {
            self.stream.advance(); // consume '.'
            while let Some(c) = self.stream.current {
                if c.is_ascii_digit() {
                    self.stream.advance();
                } else {
                    break;
                }
            }
        }
        Ok(self.make_token(TokenKind::Number))
    }

    fn read_string(&mut self) -> Result<Token> {
        self.stream.advance(); // skip "
        loop {
            if self.stream.is_eof() {
                return Err(self.error("String tidak ditutup dengan \""));
            }
            let c = self.stream.current.unwrap();
            if c == '"' {
                self.stream.advance();
                return Ok(self.make_token(TokenKind::String));
            }
            if c == '\\' {
                self.stream.advance();
                if self.stream.is_eof() {
                    return Err(self.error("Escape sequence tidak lengkap"));
                }
            }
            self.stream.advance();
        }
    }

    fn read_char(&mut self) -> Result<Token> {
        self.stream.advance(); // skip '
        if self.stream.is_eof() {
            return Err(self.error("Char tidak ditutup dengan '"));
        }
        if self.stream.current == Some('\\') {
            self.stream.advance();
        }
        self.stream.advance();
        if self.stream.is_eof() || self.stream.current != Some('\'') {
            return Err(self.error("Char tidak ditutup dengan '"));
        }
        self.stream.advance();
        Ok(self.make_token(TokenKind::CharLit))
    }

    fn read_operator(&mut self) -> Result<Token> {
        let c = self.stream.current.unwrap();

        let (kind, advance_count): (TokenKind, usize) = match c {
            '(' => (TokenKind::LParen, 1),
            ')' => (TokenKind::RParen, 1),
            '{' => (TokenKind::LBrace, 1),
            '}' => (TokenKind::RBrace, 1),
            '[' => (TokenKind::LBracket, 1),
            ']' => (TokenKind::RBracket, 1),
            ',' => (TokenKind::Comma, 1),
            ';' => (TokenKind::Semicolon, 1),
            ':' => (TokenKind::Colon, 1),
            '#' => (TokenKind::Hash, 1),
            '@' => (TokenKind::At, 1),
            '|' => (TokenKind::Pipe, 1),
            '_' => (TokenKind::Underscore, 1),

            '+' if self.stream.peek(1) == Some('+') => (TokenKind::Concat, 2),
            '+' => (TokenKind::Plus, 1),
            '-' if self.stream.peek(1) == Some('>') => (TokenKind::Arrow, 2),
            '-' => (TokenKind::Minus, 1),
            '*' => (TokenKind::Star, 1),
            '/' => (TokenKind::Slash, 1),
            '%' => (TokenKind::Percent, 1),

            '=' if self.stream.peek(1) == Some('=') => (TokenKind::Eq, 2),
            '=' if self.stream.peek(1) == Some('>') => (TokenKind::FatArrow, 2),
            '=' => (TokenKind::Assign, 1),

            '!' if self.stream.peek(1) == Some('=') => (TokenKind::Ne, 2),
            '!' => (TokenKind::Bang, 1),

            '<' if self.stream.peek(1) == Some('=') => (TokenKind::Le, 2),
            '<' => (TokenKind::Lt, 1),

            '>' if self.stream.peek(1) == Some('=') => (TokenKind::Ge, 2),
            '>' => (TokenKind::Gt, 1),

            '.' if self.stream.peek(1) == Some('.') && self.stream.peek(2) == Some('.') => (TokenKind::DotDotDot, 3),
            '.' if self.stream.peek(1) == Some('.') => (TokenKind::DotDot, 2),
            '.' => (TokenKind::Dot, 1),

            '&' if self.stream.peek(1) == Some('&') => (TokenKind::And, 2),
            '?' if self.stream.peek(1) == Some('.') => (TokenKind::Question, 2),
            '?' => (TokenKind::Question, 1),

            other => {
                return Err(Diagnostic::error(format!("Karakter tidak dikenal: '{}'", other))
                    .at(Span::new(self.start, self.stream.pos + 1, self.source_map.id())));
            }
        };

        for _ in 0..advance_count {
            self.stream.advance();
        }
        Ok(self.make_token(kind))
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.stream.current {
                Some(' ') | Some('\t') | Some('\r') => { self.stream.advance(); }
                Some('\n') => { self.stream.advance(); }
                _ => break,
            }
        }
    }

    fn skip_comment_line(&mut self) {
        while let Some(c) = self.stream.current {
            if c == '\n' {
                return;
            }
            self.stream.advance();
        }
    }

    fn skip_comment_block(&mut self) {
        // skip /*
        self.stream.advance();
        self.stream.advance();
        loop {
            let is_end = self.stream.current == Some('*') && self.stream.peek(1) == Some('/');
            if self.stream.current.is_none() {
                return;
            }
            if is_end {
                self.stream.advance();
                self.stream.advance();
                return;
            }
            self.stream.advance();
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        let start = self.start;
        let end = self.stream.pos;
        let span = Span::new(start, end, self.source_map.id());
        let lexeme = self.source[start..end].to_string();
        Token::new(kind, lexeme, span)
    }

    fn error(&self, msg: &str) -> Diagnostic {
        let start = self.start;
        let end = (self.stream.pos).max(start + 1).min(self.source.len());
        Diagnostic::error(msg).at(Span::new(start, end, self.source_map.id()))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(tok) if tok.is(TokenKind::Eof) => None,
            Ok(tok) => Some(Ok(tok)),
            Err(e) => Some(Err(e)),
        }
    }
}
