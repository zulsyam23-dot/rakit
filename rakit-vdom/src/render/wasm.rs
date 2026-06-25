use crate::node::*;

pub struct WasmRenderer;

impl WasmRenderer {
    pub fn new() -> Self {
        WasmRenderer
    }

    pub fn render_node<'a>(&self, node: &'a VDomNode) -> &'a VDomNode {
        node
    }
}
