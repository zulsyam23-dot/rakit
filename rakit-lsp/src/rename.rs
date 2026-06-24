pub struct RenameEngine;

impl RenameEngine {
    pub fn new() -> Self {
        RenameEngine
    }

    pub fn rename(&self, text: &str, old_name: &str, new_name: &str) -> String {
        text.replace(old_name, new_name)
    }
}

impl Default for RenameEngine {
    fn default() -> Self {
        Self::new()
    }
}
