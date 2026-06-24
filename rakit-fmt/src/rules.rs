#[derive(Debug, Clone, PartialEq)]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BraceStyle {
    SameLine,
    NewLine,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuoteStyle {
    Single,
    Double,
}

#[derive(Debug, Clone)]
pub struct FormatRules {
    pub indent_style: IndentStyle,
    pub indent_size: usize,
    pub max_line_length: usize,
    pub brace_style: BraceStyle,
    pub space_around_ops: bool,
    pub jsx_attrs_multiline: bool,
    pub trailing_comma: bool,
    pub quote_style: QuoteStyle,
}

impl Default for FormatRules {
    fn default() -> Self {
        FormatRules {
            indent_style: IndentStyle::Spaces,
            indent_size: 4,
            max_line_length: 100,
            brace_style: BraceStyle::SameLine,
            space_around_ops: true,
            jsx_attrs_multiline: true,
            trailing_comma: true,
            quote_style: QuoteStyle::Double,
        }
    }
}
