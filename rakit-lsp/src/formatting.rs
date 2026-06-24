pub struct FormatEngine {
    indent_size: usize,
    use_tabs: bool,
}

impl FormatEngine {
    pub fn new() -> Self {
        FormatEngine {
            indent_size: 4,
            use_tabs: false,
        }
    }

    pub fn with_indent(indent_size: usize) -> Self {
        FormatEngine {
            indent_size,
            use_tabs: false,
        }
    }

    pub fn format(&self, source: &str) -> String {
        let mut result = String::new();
        let mut indent_level = 0;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                result.push('\n');
                continue;
            }

            let deindent = trimmed.starts_with('}')
                || trimmed.starts_with(']')
                || trimmed.starts_with(')');

            if deindent && indent_level > 0 {
                indent_level -= 1;
            }

            let indent_str = if self.use_tabs {
                "\t".repeat(indent_level)
            } else {
                " ".repeat(indent_level * self.indent_size)
            };

            result.push_str(&indent_str);
            result.push_str(&self.format_line(trimmed));
            result.push('\n');

            let open_count = trimmed.chars().filter(|&c| c == '{').count();
            let close_count = trimmed.chars().filter(|&c| c == '}').count();

            if open_count > close_count {
                indent_level += open_count - close_count;
            }
        }

        result
    }

    fn format_line(&self, line: &str) -> String {
        let mut result = String::new();
        let mut prev_char: Option<char> = None;

        for ch in line.chars() {
            if prev_char == Some(',') && ch != ' ' {
                result.push(' ');
            }
            if prev_char == Some('>') && ch.is_alphanumeric() {
                result.push(' ');
            }
            result.push(ch);
            prev_char = Some(ch);
        }

        result
    }
}

impl Default for FormatEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple() {
        let formatter = FormatEngine::new();
        let input = "fungsi main() {\ncetak(\"halo\")\n}";
        let output = formatter.format(input);
        assert!(output.contains("    cetak"));
    }

    #[test]
    fn test_format_nested() {
        let formatter = FormatEngine::new();
        let input = "fungsi main() {\njika (benar) {\ncetak(\"ya\")\n}\n}";
        let output = formatter.format(input);
        assert!(output.contains("        cetak"));
    }

    #[test]
    fn test_format_empty_lines() {
        let formatter = FormatEngine::new();
        let input = "fungsi main() {\n\ncetak(\"halo\")\n\n}";
        let output = formatter.format(input);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_tabs() {
        let formatter = FormatEngine::with_indent(1);
        let input = "a {\nb {\nc\n}\n}";
        let output = formatter.format(input);
        assert!(!output.is_empty());
    }
}
