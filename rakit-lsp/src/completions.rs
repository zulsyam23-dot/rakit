use std::collections::HashMap;

pub struct CompletionEngine {
    keywords: Vec<String>,
    custom_completions: HashMap<String, Vec<String>>,
}

impl CompletionEngine {
    pub fn new() -> Self {
        CompletionEngine {
            keywords: vec![
                "fungsi".into(),
                "komponen".into(),
                "tampilkan".into(),
                "keadaan".into(),
                "efek".into(),
                "ingat".into(),
                "panggil".into(),
                "acu".into(),
                "konteks".into(),
                "pengedger".into(),
                "jalan".into(),
                "konstan".into(),
                "ubah".into(),
                "jika".into(),
                "lain".into(),
                "ulang".into(),
                "cocok".into(),
                "kasus".into(),
                "coba".into(),
                "tangkap".into(),
                "lempar".into(),
                "berhenti".into(),
                "dari".into(),
                "ke".into(),
                "tipe".into(),
                "struktur".into(),
                "pilih".into(),
                "benar".into(),
                "salah".into(),
                "batal".into(),
            ],
            custom_completions: HashMap::new(),
        }
    }

    pub fn get_keywords(&self) -> &[String] {
        &self.keywords
    }

    pub fn completions_for(&self, prefix: &str) -> Vec<String> {
        let mut results: Vec<String> = self
            .keywords
            .iter()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        for (_key, values) in &self.custom_completions {
            for val in values {
                if val.starts_with(prefix) && !results.contains(val) {
                    results.push(val.clone());
                }
            }
        }

        results.sort();
        results
    }

    pub fn add_custom_completions(&mut self, key: String, completions: Vec<String>) {
        self.custom_completions.insert(key, completions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_keywords() {
        let engine = CompletionEngine::new();
        assert!(engine.get_keywords().contains(&"fungsi".to_string()));
    }

    #[test]
    fn test_completions_for_prefix() {
        let engine = CompletionEngine::new();
        let results = engine.completions_for("fu");
        assert!(results.contains(&"fungsi".to_string()));
    }

    #[test]
    fn test_completions_empty_prefix() {
        let engine = CompletionEngine::new();
        let results = engine.completions_for("");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_completions_no_match() {
        let engine = CompletionEngine::new();
        let results = engine.completions_for("zzz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_custom_completions() {
        let mut engine = CompletionEngine::new();
        engine.add_custom_completions(
            "KomponenSaya".into(),
            vec!["props".into(), "state".into()],
        );
        let results = engine.completions_for("props");
        assert!(results.contains(&"props".to_string()));
    }
}
