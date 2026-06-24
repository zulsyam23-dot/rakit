pub struct CssomManager {
    rules: Vec<String>,
}

impl CssomManager {
    pub fn new() -> Self {
        CssomManager { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, selector: &str, properties: &[(&str, &str)]) {
        let mut css = format!("{} {{\n", selector);
        for (prop, val) in properties {
            css.push_str(&format!("  {}: {};\n", prop, val));
        }
        css.push_str("}\n");
        self.rules.push(css);
    }

    pub fn to_string(&self) -> String {
        self.rules.join("\n")
    }

    pub fn clear(&mut self) {
        self.rules.clear();
    }

    pub fn inject_into_document(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let doc = web_sys::window()
                .and_then(|w| w.document())
                .expect("No document");
            if let Some(head) = doc.head() {
                let style = doc.create_element("style").unwrap();
                style.set_text_content(Some(&self.to_string()));
                let _ = head.append_child(&style);
            }
        }
    }
}
