use crate::extract::DocItem;

#[derive(Debug, Clone)]
pub struct SearchIndex {
    pub entries: Vec<SearchEntry>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchEntry {
    pub name: String,
    pub kind: String,
    pub description: String,
    pub relevance: f64,
}

impl SearchIndex {
    pub fn new() -> Self {
        SearchIndex { entries: vec![] }
    }

    pub fn from_docs(items: &[DocItem]) -> Self {
        let entries = items.iter().map(|item| {
            let kind = match item.kind {
                crate::extract::DocItemKind::Function => "fungsi",
                crate::extract::DocItemKind::Component => "komponen",
                crate::extract::DocItemKind::Struct => "struktur",
                crate::extract::DocItemKind::Enum => "enum",
                crate::extract::DocItemKind::TypeAlias => "alias",
            };
            SearchEntry {
                name: item.name.clone(),
                kind: kind.to_string(),
                description: item.description.clone(),
                relevance: 1.0,
            }
        }).collect();

        SearchIndex { entries }
    }

    pub fn search(&self, query: &str) -> Vec<SearchEntry> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchEntry> = self.entries.iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&query_lower)
                    || e.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        results.sort_by(|a, b| {
            let a_name = a.name.to_lowercase().starts_with(&query_lower) as i32;
            let b_name = b.name.to_lowercase().starts_with(&query_lower) as i32;
            b_name.cmp(&a_name).then_with(|| {
                b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        results
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_default()
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}
