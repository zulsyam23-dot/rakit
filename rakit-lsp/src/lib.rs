pub mod completions;
pub mod diagnostics;
pub mod hover;
pub mod code_action;
pub mod references;
pub mod rename;
pub mod folding;
pub mod symbols;
pub mod formatting;

use std::collections::HashMap;
use completions::CompletionEngine;
use diagnostics::DiagnosticsEngine;
use hover::HoverEngine;
use code_action::CodeActionEngine;
use references::ReferencesEngine;
use rename::RenameEngine;
use folding::FoldingEngine;
use symbols::SymbolEngine;
use formatting::FormatEngine;

#[derive(Debug, Clone)]
pub struct DocumentState {
    pub uri: String,
    pub text: String,
    pub version: i32,
}

impl DocumentState {
    pub fn new(uri: String, text: String) -> Self {
        DocumentState { uri, text, version: 1 }
    }

    pub fn update(&mut self, text: String, version: i32) {
        self.text = text;
        self.version = version;
    }

    pub fn line_count(&self) -> usize {
        self.text.lines().count()
    }

    pub fn line_text(&self, line: u32) -> &str {
        self.text.lines().nth(line as usize).unwrap_or("")
    }

    pub fn word_at_position(&self, line: u32, column: u32) -> Option<String> {
        let line_text = self.line_text(line);
        if (column as usize) >= line_text.len() {
            return None;
        }
        let col = column as usize;
        let chars: Vec<char> = line_text.chars().collect();

        let start = chars[..=col].iter().rposition(|c| !c.is_alphanumeric() && *c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = chars[col..].iter().position(|c| !c.is_alphanumeric() && *c != '_')
            .map(|i| col + i)
            .unwrap_or(chars.len());

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }
}

pub struct LspServer {
    pub completions: CompletionEngine,
    pub diagnostics: DiagnosticsEngine,
    pub hover: HoverEngine,
    pub code_actions: CodeActionEngine,
    pub references: ReferencesEngine,
    pub rename: RenameEngine,
    pub folding: FoldingEngine,
    pub symbols: SymbolEngine,
    pub formatter: FormatEngine,
    pub documents: HashMap<String, DocumentState>,
}

impl LspServer {
    pub fn new() -> Self {
        LspServer {
            completions: CompletionEngine::new(),
            diagnostics: DiagnosticsEngine::new(),
            hover: HoverEngine::new(),
            code_actions: CodeActionEngine::new(),
            references: ReferencesEngine::new(),
            rename: RenameEngine::new(),
            folding: FoldingEngine::new(),
            symbols: SymbolEngine::new(),
            formatter: FormatEngine::new(),
            documents: HashMap::new(),
        }
    }

    pub fn open_document(&mut self, uri: String, source: String) {
        let state = DocumentState::new(uri.clone(), source.clone());
        self.documents.insert(uri.clone(), state);
        self.diagnostics.update_diagnostics(&uri, &source);
    }

    pub fn close_document(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    pub fn update_document(&mut self, uri: &str, source: String, version: i32) {
        if let Some(state) = self.documents.get_mut(uri) {
            state.update(source.clone(), version);
        } else {
            let state = DocumentState::new(uri.to_string(), source.clone());
            self.documents.insert(uri.to_string(), state);
        }
        self.diagnostics.update_diagnostics(uri, &source);
    }

    pub fn get_document(&self, uri: &str) -> Option<&DocumentState> {
        self.documents.get(uri)
    }

    pub fn get_completions(&self, uri: &str, line: u32, column: u32) -> Vec<String> {
        if let Some(doc) = self.documents.get(uri) {
            let line_text = doc.line_text(line);
            let prefix: String = line_text.chars()
                .take(column as usize)
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            self.completions.completions_for(&prefix)
        } else {
            vec![]
        }
    }

    pub fn get_hover(&self, uri: &str, line: u32, column: u32) -> Option<String> {
        if let Some(doc) = self.documents.get(uri) {
            if let Some(word) = doc.word_at_position(line, column) {
                return self.hover.hover_for(&word).map(|s| s.to_string());
            }
        }
        None
    }

    pub fn get_code_actions(&self, uri: &str, line: u32) -> Vec<String> {
        if let Some(doc) = self.documents.get(uri) {
            let line_text = doc.line_text(line);
            self.code_actions.actions_for(uri, line, line_text)
        } else {
            vec![]
        }
    }

    pub fn find_references(&self, uri: &str, word: &str) -> Vec<(String, u32, u32)> {
        self.references.find_references(&self.documents, uri, word)
    }

    pub fn rename_symbol(&self, uri: &str, line: u32, column: u32, new_name: &str) -> Option<String> {
        if let Some(doc) = self.documents.get(uri) {
            if let Some(word) = doc.word_at_position(line, column) {
                return Some(self.rename.rename(&doc.text, &word, new_name));
            }
        }
        None
    }

    pub fn get_folding_ranges(&self, uri: &str) -> Vec<(u32, u32)> {
        if let Some(doc) = self.documents.get(uri) {
            self.folding.folding_ranges(&doc.text)
        } else {
            vec![]
        }
    }

    pub fn get_document_symbols(&self, uri: &str) -> Vec<(String, String, u32)> {
        if let Some(doc) = self.documents.get(uri) {
            self.symbols.symbols_in(&doc.text)
        } else {
            vec![]
        }
    }

    pub fn format_document(&self, uri: &str) -> Option<String> {
        if let Some(doc) = self.documents.get(uri) {
            Some(self.formatter.format(&doc.text))
        } else {
            None
        }
    }

    pub fn get_diagnostics(&self, _uri: &str) -> Vec<diagnostics::Diagnostic> {
        self.diagnostics.diagnostics.clone()
    }

    pub fn all_keywords(&self) -> &[String] {
        self.completions.get_keywords()
    }
}

impl Default for LspServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_server_open_close() {
        let mut server = LspServer::new();
        server.open_document("file.rakit".into(), "fungsi main() {}".into());
        assert!(server.documents.contains_key("file.rakit"));
        server.close_document("file.rakit");
        assert!(!server.documents.contains_key("file.rakit"));
    }

    #[test]
    fn test_lsp_server_update() {
        let mut server = LspServer::new();
        server.open_document("file.rakit".into(), "fungsi main() {}".into());
        server.update_document("file.rakit", "komponen App() { tampilkan { <div/> } }".into(), 2);
        assert_eq!(server.documents.get("file.rakit").unwrap().version, 2);
    }

    #[test]
    fn test_lsp_server_completions() {
        let mut server = LspServer::new();
        server.open_document("file.rakit".into(), "fu".into());
        let comps = server.get_completions("file.rakit", 0, 2);
        assert!(comps.iter().any(|c| c == "fungsi"));
    }

    #[test]
    fn test_lsp_server_hover() {
        let server = LspServer::new();
        let hover = server.get_hover("file.rakit", 0, 0);
        assert!(hover.is_none()); // doc not open
    }

    #[test]
    fn test_lsp_server_format() {
        let mut server = LspServer::new();
        server.open_document("file.rakit".into(), "  fungsi   main()  {  }".into());
        let formatted = server.format_document("file.rakit");
        assert!(formatted.is_some());
    }

    #[test]
    fn test_lsp_server_symbols() {
        let mut server = LspServer::new();
        server.open_document("file.rakit".into(), "fungsi main() {}\nkomponen App() { tampilkan { <div/> } }".into());
        let symbols = server.get_document_symbols("file.rakit");
        assert!(symbols.iter().any(|(name, _, _)| name == "main"));
        assert!(symbols.iter().any(|(name, _, _)| name == "App"));
    }

    #[test]
    fn test_lsp_server_folding() {
        let mut server = LspServer::new();
        server.open_document("file.rakit".into(), "fungsi main() {\n  cetak(\"halo\")\n}".into());
        let folds = server.get_folding_ranges("file.rakit");
        assert!(!folds.is_empty());
    }

    #[test]
    fn test_word_at_position() {
        let state = DocumentState::new("test.rakit".into(), "fungsi main()".into());
        assert_eq!(state.word_at_position(0, 0), Some("fungsi".into()));
        assert_eq!(state.word_at_position(0, 7), Some("main".into()));
    }

    #[test]
    fn test_line_text() {
        let state = DocumentState::new("test.rakit".into(), "baris 1\nbaris 2".into());
        assert_eq!(state.line_text(0), "baris 1");
        assert_eq!(state.line_text(1), "baris 2");
        assert_eq!(state.line_text(99), "");
    }
}
