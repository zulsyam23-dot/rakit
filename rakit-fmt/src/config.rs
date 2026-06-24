use crate::rules::FormatRules;

pub struct FormatConfig {
    pub rules: FormatRules,
}

impl FormatConfig {
    pub fn new() -> Self {
        FormatConfig {
            rules: FormatRules::default(),
        }
    }

    pub fn load_from_str(_json: &str) -> Result<Self, String> {
        // TODO: Parse .rakit.json config
        Ok(FormatConfig::new())
    }

    pub fn load_default() -> Self {
        FormatConfig::new()
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = FormatConfig::default();
        assert_eq!(config.rules.indent_size, 4);
    }

    #[test]
    fn test_config_load_default() {
        let config = FormatConfig::load_default();
        assert_eq!(config.rules.max_line_length, 100);
    }
}
