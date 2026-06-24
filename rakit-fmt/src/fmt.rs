use crate::rules::{BraceStyle, FormatRules};
use std::fmt::Write;

pub struct RakitFormatter {
    pub rules: FormatRules,
    output: String,
    indent_level: usize,
}

impl RakitFormatter {
    pub fn new(rules: FormatRules) -> Self {
        RakitFormatter {
            rules,
            output: String::new(),
            indent_level: 0,
        }
    }

    pub fn format(&mut self, source: &str) -> Result<String, String> {
        self.output.clear();
        self.indent_level = 0;

        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() {
                self.output.push('\n');
                i += 1;
                continue;
            }
            self.format_line(line);
            self.output.push('\n');
            i += 1;
        }

        Ok(self.output.clone())
    }

    fn format_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.starts_with('}') || trimmed == "}" {
            self.indent_level = self.indent_level.saturating_sub(1);
        }

        self.write_indent();

        if trimmed.starts_with("//") {
            self.output.push_str(trimmed);
        } else if trimmed.ends_with('{') {
            let content = trimmed.trim_end_matches('{').trim();
            self.output.push_str(content);
            match self.rules.brace_style {
                BraceStyle::SameLine => self.output.push_str(" {"),
                BraceStyle::NewLine => {
                    self.output.push('\n');
                    self.write_indent();
                    self.output.push('{');
                }
            }
            self.indent_level += 1;
        } else if let Some(pos) = trimmed.find('{') {
            let before_brace = trimmed[..pos].trim();
            let after_brace = trimmed[pos..].trim_start_matches('{').trim();
            self.output.push_str(before_brace);
            self.output.push_str(" { ");
            self.output.push_str(after_brace);
            if trimmed.ends_with('}') {
                self.indent_level = self.indent_level.saturating_sub(1);
            }
        } else {
            self.output.push_str(trimmed);
        }
    }

    pub fn format_function(&mut self, name: &str, params: &[(&str, &str)], body: &[&str]) {
        self.write_indent();
        write!(self.output, "fungsi {}(", name);
        for (i, (p_name, p_type)) in params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            write!(self.output, "{}: {}", p_name, p_type);
        }
        self.output.push_str(") {");
        self.output.push('\n');

        self.indent_level += 1;
        for stmt in body {
            self.write_indent();
            self.output.push_str(stmt);
            self.output.push('\n');
        }
        self.indent_level -= 1;

        self.write_indent();
        self.output.push('}');
        self.output.push('\n');
    }

    pub fn format_component(
        &mut self,
        name: &str,
        props: &[(&str, &str)],
        statements: &[&str],
        render_expr: &str,
    ) {
        self.write_indent();
        write!(self.output, "komponen {}(", name);
        for (i, (p_name, p_type)) in props.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            write!(self.output, "{}: {}", p_name, p_type);
        }
        self.output.push_str(") {");
        self.output.push('\n');

        self.indent_level += 1;
        for stmt in statements {
            self.write_indent();
            self.output.push_str(stmt);
            self.output.push('\n');
        }

        self.write_indent();
        self.output.push_str("tampilkan {");
        self.output.push('\n');
        self.indent_level += 1;
        self.write_indent();
        self.output.push_str(render_expr);
        self.output.push('\n');
        self.indent_level -= 1;
        self.write_indent();
        self.output.push('}');
        self.output.push('\n');

        self.indent_level -= 1;
        self.write_indent();
        self.output.push('}');
        self.output.push('\n');
    }

    pub fn format_jsx(&mut self, tag: &str, attrs: &[(&str, &str)], children: &[&str]) {
        write!(self.output, "<{}", tag);

        if attrs.is_empty() {
        } else if attrs.len() == 1 || !self.rules.jsx_attrs_multiline {
            for (k, v) in attrs {
                write!(self.output, " {}=\"{}\"", k, v);
            }
        } else {
            self.output.push('\n');
            self.indent_level += 1;
            for (k, v) in attrs {
                self.write_indent();
                write!(self.output, "{}=\"{}\"", k, v);
                self.output.push('\n');
            }
            self.indent_level -= 1;
            self.write_indent();
        }

        if children.is_empty() {
            self.output.push_str(" />");
            return;
        }

        self.output.push('>');
        self.output.push('\n');
        self.indent_level += 1;
        for child in children {
            self.write_indent();
            self.output.push_str(child);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.write_indent();
        write!(self.output, "</{}>", tag);
    }

    fn write_indent(&mut self) {
        match self.rules.indent_style {
            crate::rules::IndentStyle::Spaces => {
                for _ in 0..self.indent_level * self.rules.indent_size {
                    self.output.push(' ');
                }
            }
            crate::rules::IndentStyle::Tabs => {
                for _ in 0..self.indent_level {
                    self.output.push('\t');
                }
            }
        }
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }
}

impl std::fmt::Write for RakitFormatter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.output.push_str(s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::FormatRules;

    #[test]
    fn test_formatter_function_roundtrip() {
        let mut formatter = RakitFormatter::new(FormatRules::default());
        let input = "fungsi main() -> I32{42}";
        let output = formatter.format(input).unwrap();
        assert!(output.contains("fungsi main() -> I32 {"));
    }

    #[test]
    fn test_formatter_basic_indent() {
        let mut formatter = RakitFormatter::new(FormatRules::default());
        let input = "komponen HaloDunia() {\ntampilkan {\n<div/>\n}\n}";
        let output = formatter.format(input).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_function_api() {
        let mut formatter = RakitFormatter::new(FormatRules::default());
        formatter.format_function(
            "tambah",
            &[("a", "I32"), ("b", "I32")],
            &["a + b"],
        );
        let output = formatter.get_output();
        assert!(output.contains("fungsi tambah(a: I32, b: I32) {"));
        assert!(output.contains("a + b"));
    }

    #[test]
    fn test_format_component_api() {
        let mut formatter = RakitFormatter::new(FormatRules::default());
        formatter.format_component(
            "Tombol",
            &[("teks", "String")],
            &["keadaan(hitung, aturHitung) = 0;"],
            "<button>{teks}</button>",
        );
        let output = formatter.get_output();
        assert!(output.contains("komponen Tombol(teks: String) {"));
        assert!(output.contains("tampilkan"));
    }

    #[test]
    fn test_format_jsx_empty() {
        let mut formatter = RakitFormatter::new(FormatRules::default());
        formatter.format_jsx("br", &[], &[]);
        assert_eq!(formatter.get_output(), "<br />");
    }

    #[test]
    fn test_format_jsx_with_children() {
        let mut formatter = RakitFormatter::new(FormatRules::default());
        formatter.format_jsx("div", &[("class", "container")], &["Halo", "Dunia"]);
        let output = formatter.get_output();
        assert!(output.contains("<div"));
        assert!(output.contains("</div>"));
        assert!(output.contains("Halo"));
    }

    #[test]
    fn test_format_preserves_comments() {
        let mut formatter = RakitFormatter::new(FormatRules::default());
        let input = "// ini komentar\nfungsi main() {}";
        let output = formatter.format(input).unwrap();
        assert!(output.contains("// ini komentar"));
    }
}
