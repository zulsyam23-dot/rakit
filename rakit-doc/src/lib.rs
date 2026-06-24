pub mod extract;
pub mod render;
pub mod search;

use extract::DocItem;
use rakit_ir_hir::hir::HirProgram;

pub struct DocGenerator {
    pub items: Vec<DocItem>,
}

impl DocGenerator {
    pub fn new() -> Self {
        DocGenerator { items: vec![] }
    }

    pub fn from_program(program: &HirProgram) -> Self {
        let items = extract::extract_docs_from_hir(program);
        DocGenerator { items }
    }

    pub fn generate_html(&self) -> String {
        render::render_html(&self.items)
    }

    pub fn generate_markdown(&self) -> String {
        render::render_markdown(&self.items)
    }
}

impl Default for DocGenerator {
    fn default() -> Self {
        Self::new()
    }
}
