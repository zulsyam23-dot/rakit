#[allow(dead_code)]
pub struct GtkCssManager {
    provider_id: u32,
}

impl GtkCssManager {
    pub fn new() -> Self {
        GtkCssManager { provider_id: 0 }
    }

    pub fn apply_css(&self, css: &str) -> String {
        let mut wrapped = String::from("/* Rakit GTK4 CSS */\n");
        wrapped.push_str(css);
        wrapped
    }

    pub fn wrap_css(tag: &str, properties: &[(&str, &str)]) -> String {
        let mut css = format!("{} {{\n", tag);
        for (prop, val) in properties {
            css.push_str(&format!("  {}: {};\n", prop, val));
        }
        css.push_str("}\n");
        css
    }

    pub fn heading_css() -> String {
        String::from(
            ".heading-1 { font-size: 32px; font-weight: bold; }\n\
             .heading-2 { font-size: 28px; font-weight: bold; }\n\
             .heading-3 { font-size: 24px; font-weight: bold; }\n\
             .heading-4 { font-size: 20px; font-weight: bold; }\n\
             .heading-5 { font-size: 18px; font-weight: bold; }\n\
             .heading-6 { font-size: 16px; font-weight: bold; }\n",
        )
    }
}
