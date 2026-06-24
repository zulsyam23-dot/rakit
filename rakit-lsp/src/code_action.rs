use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub kind: String,
    pub edit: Option<String>,
}

pub struct CodeActionEngine {
    actions: HashMap<String, Vec<CodeAction>>,
    diagnostics_context: Vec<String>,
}

impl CodeActionEngine {
    pub fn new() -> Self {
        CodeActionEngine {
            actions: HashMap::new(),
            diagnostics_context: vec![
                "Kurung kurawal tidak ditutup".to_string(),
                "Tipe tidak cocok".to_string(),
                "Variabel tidak dikenal".to_string(),
            ],
        }
    }

    pub fn actions_for(&self, _uri: &str, _line: u32, line_text: &str) -> Vec<String> {
        let mut results = Vec::new();

        if line_text.trim().ends_with('{') && !line_text.trim().ends_with("=>") {
            results.push("Tutup block: tambahkan '}'".into());
        }

        if line_text.trim().starts_with("fungsi") || line_text.trim().starts_with("komponen") {
            if !line_text.contains('{') {
                results.push("Generate function body".into());
            }
        }

        if line_text.contains("==") && !line_text.contains("!=") {
            results.push("Convert == to != (negate condition)".into());
        }

        if line_text.contains("cetak(") {
            results.push("Add import 'dari \"rakit/cetak\" ambil { cetak }'".into());
        }

        results
    }

    pub fn add_action(&mut self, uri: String, action: CodeAction) {
        self.actions.entry(uri).or_default().push(action);
    }
}

impl Default for CodeActionEngine {
    fn default() -> Self {
        Self::new()
    }
}
