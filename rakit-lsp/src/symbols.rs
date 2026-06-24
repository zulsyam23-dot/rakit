pub struct SymbolEngine;

impl SymbolEngine {
    pub fn new() -> Self {
        SymbolEngine
    }

    pub fn symbols_in(&self, text: &str) -> Vec<(String, String, u32)> {
        let mut symbols = Vec::new();

        for (i, line) in text.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("fungsi ") {
                let name = trimmed.trim_start_matches("fungsi ")
                    .split(&['(', ' ', '{'][..])
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    symbols.push((name, "function".into(), i as u32));
                }
            } else if trimmed.starts_with("komponen ") {
                let name = trimmed.trim_start_matches("komponen ")
                    .split(&['(', ' ', '{'][..])
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    symbols.push((name, "component".into(), i as u32));
                }
            } else if trimmed.starts_with("struktur ") {
                let name = trimmed.trim_start_matches("struktur ")
                    .split(&[' ', '{'][..])
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    symbols.push((name, "struct".into(), i as u32));
                }
            } else if trimmed.starts_with("pilih ") {
                let name = trimmed.trim_start_matches("pilih ")
                    .split(&[' ', '{'][..])
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    symbols.push((name, "enum".into(), i as u32));
                }
            } else if trimmed.starts_with("tipe ") {
                let name = trimmed.trim_start_matches("tipe ")
                    .split(&[' ', '='][..])
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    symbols.push((name, "type".into(), i as u32));
                }
            }
        }

        symbols
    }
}

impl Default for SymbolEngine {
    fn default() -> Self {
        Self::new()
    }
}
