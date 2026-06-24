#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

pub struct DiagnosticsEngine {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticsEngine {
    pub fn new() -> Self {
        DiagnosticsEngine {
            diagnostics: Vec::new(),
        }
    }

    pub fn update_diagnostics(&mut self, _uri: &str, source: &str) {
        self.diagnostics.clear();
        let lines: Vec<&str> = source.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            if let Some(col) = self.check_unclosed_brace(trimmed) {
                self.diagnostics.push(Diagnostic {
                    message: "Kurung kurawal tidak ditutup".into(),
                    line: line_num + 1,
                    column: col,
                    severity: DiagnosticSeverity::Error,
                });
            }
        }
    }

    fn check_unclosed_brace(&self, line: &str) -> Option<usize> {
        let open_count = line.chars().filter(|&c| c == '{').count();
        let close_count = line.chars().filter(|&c| c == '}').count();
        if open_count > close_count {
            line.find('{').map(|i| i + 1)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_no_errors() {
        let mut engine = DiagnosticsEngine::new();
        engine.update_diagnostics("test.rakit", "fungsi main() {}");
        assert!(engine.diagnostics.is_empty());
    }

    #[test]
    fn test_diagnostics_unclosed_brace() {
        let mut engine = DiagnosticsEngine::new();
        engine.update_diagnostics("test.rakit", "komponen App() {\n");
        assert!(!engine.diagnostics.is_empty());
    }

    #[test]
    fn test_diagnostics_clear() {
        let mut engine = DiagnosticsEngine::new();
        engine.update_diagnostics("test.rakit", "komponen App() {\n");
        assert!(!engine.diagnostics.is_empty());
        engine.clear();
        assert!(engine.diagnostics.is_empty());
    }
}
