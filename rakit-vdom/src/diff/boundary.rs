use crate::diff::Differ;
use crate::node::VDomNode;

#[allow(dead_code)]
impl Differ {
    pub fn diff_boundary_nodes(
        &mut self,
        old: Option<&VDomNode>,
        new: &VDomNode,
        path: &[usize],
        _boundary_id: u64,
    ) {
        if is_error_fallback(new) {
            self.diff_children(
                &old.map(|o| vec![o.clone()]).unwrap_or_default(),
                &[new.clone()],
                path,
                None,
            );
        } else {
            self.diff_node(old, new, &mut path.to_vec());
        }
    }
}

#[allow(dead_code)]
fn is_error_fallback(node: &VDomNode) -> bool {
    match node {
        VDomNode::Element(e) => {
            e.attrs.contains_key("data-rakit-error") || e.tag == "error"
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::diff;
    use crate::node::{AttrValue, ElementNode, VDomNode};
    use std::collections::HashMap;

    #[test]
    fn test_is_error_fallback_true_when_has_data_attr() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "data-rakit-error".to_string(),
            AttrValue::String("true".to_string()),
        );
        let node = VDomNode::Element(ElementNode {
            tag: "div".to_string(),
            attrs,
            events: HashMap::new(),
            children: vec![],
            key: None,
        });
        assert!(is_error_fallback(&node));
    }

    #[test]
    fn test_is_error_fallback_true_when_tag_is_error() {
        let node = VDomNode::Element(ElementNode {
            tag: "error".to_string(),
            attrs: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            key: None,
        });
        assert!(is_error_fallback(&node));
    }

    #[test]
    fn test_is_error_fallback_false_for_normal_element() {
        let node = VDomNode::element("div", vec![], vec![]);
        assert!(!is_error_fallback(&node));
    }

    #[test]
    fn test_error_boundary_transition_preserves_parent() {
        let old = VDomNode::element("main", vec![], vec![VDomNode::text("Konten normal")]);
        let new = VDomNode::element(
            "main",
            vec![],
            vec![VDomNode::Element(ElementNode {
                tag: "error".to_string(),
                attrs: HashMap::new(),
                events: HashMap::new(),
                children: vec![VDomNode::text("Terjadi error")],
                key: None,
            })],
        );

        let result = diff(Some(&old), &new);
        let parent_replaced = result
            .patches
            .iter()
            .any(|p| matches!(p, crate::diff::Patch::Replace { old_path, .. } if old_path.is_empty()));

        let child_replaced = result
            .patches
            .iter()
            .any(|p| matches!(p, crate::diff::Patch::Replace { old_path, .. } if old_path == &[0]));

        assert!(!parent_replaced, "Should not replace parent on error transition");
        assert!(child_replaced, "Child should be replaced with error fallback");
    }
}
