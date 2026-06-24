pub mod diff;
pub mod h;
pub mod node;
pub mod patch;
pub mod render;
pub mod utils;

pub use diff::{DiffResult, DiffStats, Patch, diff};
pub use h::{fragment, h, h_with_attrs, h_with_key, text, HChild};
pub use node::{
    AttrValue, ComponentNode, ElementNode, EventType, FragmentNode, TextNode, VDomNode,
};
pub use patch::native::{NativeHandle, NativePatchApplicator};
pub use render::native::NativeRenderer;
pub use render::ssr::SsrRenderer;
pub use utils::{escape_html, get_node_key, props_changed};
