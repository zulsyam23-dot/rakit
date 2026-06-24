use std::fmt;

/// Posisi dalam source code: baris dan kolom (1-indexed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SourceLoc {
    pub line: usize,
    pub column: usize,
}

impl SourceLoc {
    pub const fn new(line: usize, column: usize) -> Self {
        SourceLoc { line, column }
    }
}

impl fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Sebuah rentang dalam source code — dari byte offset start ke end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source_id: u32,
}

impl Default for Span {
    fn default() -> Self {
        Span { start: 0, end: 0, source_id: 0 }
    }
}

impl Span {
    pub const fn new(start: usize, end: usize, source_id: u32) -> Self {
        Span { start, end, source_id }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            source_id: self.source_id,
        }
    }

    pub fn empty(source_id: u32) -> Span {
        Span { start: 0, end: 0, source_id }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Map dari byte offset ke lokasi baris/kolom dalam file.
#[derive(Debug, Clone)]
pub struct SourceMap {
    pub file_name: String,
    pub source: String,
    /// line_starts[i] = byte offset dari awal baris ke-i (0-indexed)
    line_starts: Vec<usize>,
    id: u32,
}

static NEXT_SOURCE_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

impl SourceMap {
    pub fn new(file_name: &str, source: &str) -> Self {
        let id = NEXT_SOURCE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(source.char_indices()
                .filter(|(_, c)| *c == '\n')
                .map(|(i, _)| i + 1))
            .collect();

        SourceMap {
            file_name: file_name.to_string(),
            source: source.to_string(),
            line_starts,
            id,
        }
    }

    pub fn dummy() -> Self {
        SourceMap::new("<dummy>", "")
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    /// Dapatkan lokasi baris/kolom dari byte offset.
    pub fn location(&self, offset: usize) -> SourceLoc {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let line_start = self.line_starts[line];
        let column = offset - line_start;
        SourceLoc::new(line + 1, column + 1)
    }

    /// Ambil teks source untuk sebuah span.
    pub fn slice(&self, span: Span) -> &str {
        &self.source[span.start..span.end]
    }
}

/// Severity dari diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Note => write!(f, "note"),
            Severity::Help => write!(f, "help"),
        }
    }
}

/// Sebuah pesan diagnostic — error, warning, atau note.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(msg: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            message: msg.into(),
            span: Span::empty(0),
            notes: Vec::new(),
        }
    }

    pub fn warning(msg: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            message: msg.into(),
            span: Span::empty(0),
            notes: Vec::new(),
        }
    }

    pub fn at(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

/// Tipe Result khusus untuk compiler — mengumpulkan semua diagnostic.
pub type Result<T> = std::result::Result<T, Vec<Diagnostic>>;

/// Report helper — mencetak diagnostic ke stderr dengan format yang rapi.
pub fn report_diagnostics(source_map: &SourceMap, diagnostics: &[Diagnostic]) {
    let err_count = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
    let warn_count = diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();

    for diag in diagnostics {
        let loc = source_map.location(diag.span.start);
        eprintln!(
            "  {}[{}] {}",
            match diag.severity {
                Severity::Error => "\x1b[31m",
                Severity::Warning => "\x1b[33m",
                Severity::Note => "\x1b[36m",
                Severity::Help => "\x1b[32m",
            },
            diag.severity,
            diag.message,
        );

        if diag.span.start != diag.span.end || diag.span.source_id != 0 {
            let _end_loc = source_map.location(diag.span.end);
            eprintln!(
                "   \x1b[90m--> {}:{}:{}\x1b[0m",
                source_map.file_name, loc.line, loc.column
            );

            // Print source line with caret
            let line_start = source_map.line_starts.get(loc.line - 1).copied().unwrap_or(0);
            let line_end = source_map.source[line_start..]
                .find('\n')
                .map(|i| line_start + i)
                .unwrap_or(source_map.source.len());
            let line_text = &source_map.source[line_start..line_end];
            eprintln!("   \x1b[90m{:>4} |\x1b[0m {}", loc.line, line_text);

            let caret_start = diag.span.start.saturating_sub(line_start);
            let caret_end = diag.span.end.saturating_sub(line_start).max(caret_start + 1);
            eprintln!(
                "   \x1b[90m     |\x1b[0m {}{}",
                " ".repeat(caret_start),
                "\x1b[31m^\x1b[0m".repeat(caret_end - caret_start),
            );
        }

        for note in &diag.notes {
            eprintln!("   \x1b[36m= {}\x1b[0m", note);
        }
        eprintln!();
    }

    if err_count > 0 || warn_count > 0 {
        eprintln!(
            "\x1b[1m{}\x1b[0m",
            if err_count > 0 {
                format!("Build gagal: {} error, {} warning", err_count, warn_count)
            } else {
                format!("Build sukses dengan {} warning", warn_count)
            }
        );
    }
}
