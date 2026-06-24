use std::collections::HashMap;
use crate::DocumentState;

pub struct ReferencesEngine;

impl ReferencesEngine {
    pub fn new() -> Self {
        ReferencesEngine
    }

    pub fn find_references(
        &self,
        documents: &HashMap<String, DocumentState>,
        _uri: &str,
        word: &str,
    ) -> Vec<(String, u32, u32)> {
        let mut results = Vec::new();

        for (doc_uri, doc) in documents {
            for (line_num, line) in doc.text.lines().enumerate() {
                let mut col = 0;
                for token in line.split_whitespace() {
                    let clean = token.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '_')
                        .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if clean == word {
                        if let Some(pos) = line[col..].find(token) {
                            let actual_col = col + pos;
                            results.push((doc_uri.clone(), line_num as u32, actual_col as u32));
                        }
                    }
                    col += token.len() + 1;
                }
            }
        }

        results
    }
}

impl Default for ReferencesEngine {
    fn default() -> Self {
        Self::new()
    }
}
