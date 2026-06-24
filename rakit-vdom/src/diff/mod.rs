pub mod boundary;
pub mod children;
pub mod element;

use crate::node::*;

#[derive(Debug, Clone)]
pub enum Patch {
    Replace {
        old_path: Vec<usize>,
        new_node: VDomNode,
    },
    Remove {
        path: Vec<usize>,
    },
    Insert {
        parent_path: Vec<usize>,
        index: usize,
        node: VDomNode,
    },
    Move {
        from_path: Vec<usize>,
        to_parent: Vec<usize>,
        to_index: usize,
    },
    SetAttr {
        path: Vec<usize>,
        name: String,
        value: AttrValue,
    },
    RemoveAttr {
        path: Vec<usize>,
        name: String,
    },
    SetText {
        path: Vec<usize>,
        text: String,
    },
    AttachEvent {
        path: Vec<usize>,
        event_type: EventType,
        handler_id: u64,
    },
    DetachEvent {
        path: Vec<usize>,
        handler_id: u64,
    },
}

#[derive(Debug, Default)]
pub struct DiffStats {
    pub nodes_replaced: usize,
    pub nodes_moved: usize,
    pub attrs_changed: usize,
    pub events_changed: usize,
    pub text_changed: usize,
}

#[derive(Debug)]
pub struct DiffResult {
    pub patches: Vec<Patch>,
    pub stats: DiffStats,
}

pub fn diff(old: Option<&VDomNode>, new: &VDomNode) -> DiffResult {
    let mut differ = Differ::new();
    differ.diff_node(old, new, &mut vec![]);
    DiffResult {
        patches: differ.patches,
        stats: differ.stats,
    }
}

pub(crate) struct Differ {
    pub patches: Vec<Patch>,
    pub stats: DiffStats,
}

impl Differ {
    pub fn new() -> Self {
        Differ {
            patches: Vec::new(),
            stats: DiffStats::default(),
        }
    }

    pub fn diff_node(&mut self, old: Option<&VDomNode>, new: &VDomNode, path: &mut Vec<usize>) {
        match (old, new) {
            (None, new_node) => {
                let parent_path = if path.len() <= 1 {
                    vec![]
                } else {
                    path[..path.len() - 1].to_vec()
                };
                self.patches.push(Patch::Insert {
                    parent_path,
                    index: *path.last().unwrap_or(&0),
                    node: new_node.clone(),
                });
            }

            (Some(old_node), new_node) if !old_node.is_same_type(new_node) => {
                self.stats.nodes_replaced += 1;
                self.patches.push(Patch::Replace {
                    old_path: path.clone(),
                    new_node: new_node.clone(),
                });
            }

            (Some(VDomNode::Element(old_elem)), VDomNode::Element(new_elem)) => {
                self.diff_elements(old_elem, new_elem, path);
            }
            (Some(VDomNode::Text(old_text)), VDomNode::Text(new_text)) => {
                if old_text.value != new_text.value {
                    self.stats.text_changed += 1;
                    self.patches.push(Patch::SetText {
                        path: path.clone(),
                        text: new_text.value.clone(),
                    });
                }
            }
            (Some(VDomNode::Fragment(old_frag)), VDomNode::Fragment(new_frag)) => {
                self.diff_children(&old_frag.children, &new_frag.children, path, None);
            }
            (Some(VDomNode::Component(old_comp)), VDomNode::Component(new_comp)) => {
                self.diff_components(old_comp, new_comp, path);
            }

            (Some(VDomNode::Empty), VDomNode::Empty) => {}

            (Some(old_node), VDomNode::Fragment(new_frag)) => {
                let old_wrapped = vec![old_node.clone()];
                self.diff_children(&old_wrapped, &new_frag.children, path, None);
            }
            (Some(VDomNode::Fragment(old_frag)), new_node) => {
                let new_wrapped = vec![new_node.clone()];
                self.diff_children(&old_frag.children, &new_wrapped, path, None);
            }

            _ => {
                self.patches.push(Patch::Replace {
                    old_path: path.clone(),
                    new_node: new.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::VDomNode;

    #[test]
    fn test_diff_text_changed() {
        let old = VDomNode::Text(TextNode { value: "Halo".into() });
        let new = VDomNode::Text(TextNode { value: "Dunia".into() });
        let result = diff(Some(&old), &new);
        assert_eq!(result.patches.len(), 1);
        match &result.patches[0] {
            Patch::SetText { text, .. } => assert_eq!(text, "Dunia"),
            _ => panic!("expected SetText"),
        }
    }

    #[test]
    fn test_diff_element_replace_on_tag_change() {
        let old = VDomNode::element("div", vec![], vec![]);
        let new = VDomNode::element("span", vec![], vec![]);
        let result = diff(Some(&old), &new);
        match &result.patches[0] {
            Patch::Replace { new_node, .. } => {
                assert_eq!(new_node.tag(), Some("span"));
            }
            _ => panic!("expected Replace"),
        }
    }

    #[test]
    fn test_minimal_patches_for_single_attr_change() {
        let old = VDomNode::element("div", vec![("class", "old")], vec![]);
        let new = VDomNode::element("div", vec![("class", "new")], vec![]);
        let result = diff(Some(&old), &new);
        assert_eq!(result.patches.len(), 1);
        match &result.patches[0] {
            Patch::SetAttr { name, value, .. } => {
                assert_eq!(name, "class");
                assert_eq!(
                    value,
                    &AttrValue::String("new".to_string())
                );
            }
            _ => panic!("expected SetAttr"),
        }
    }

    #[test]
    fn test_empty_to_element() {
        let old = VDomNode::Empty;
        let new = VDomNode::element("div", vec![], vec![VDomNode::text("Baru")]);
        let result = diff(Some(&old), &new);
        match &result.patches[0] {
            Patch::Replace { new_node, .. } => {
                assert_eq!(new_node.tag(), Some("div"));
            }
            _ => {}
        }
    }

    #[test]
    fn test_fragment_flattening() {
        let fragment = VDomNode::Fragment(FragmentNode {
            children: vec![VDomNode::text("A"), VDomNode::text("B")],
        });
        let flat = fragment.flatten_fragments();
        assert_eq!(flat.len(), 2);
    }

    #[test]
    fn test_keyed_children_move() {
        let old_children = vec![
            VDomNode::Element(ElementNode {
                tag: "div".into(),
                attrs: Default::default(),
                events: Default::default(),
                children: vec![],
                key: Some("a".into()),
            }),
            VDomNode::Element(ElementNode {
                tag: "div".into(),
                attrs: Default::default(),
                events: Default::default(),
                children: vec![],
                key: Some("b".into()),
            }),
            VDomNode::Element(ElementNode {
                tag: "div".into(),
                attrs: Default::default(),
                events: Default::default(),
                children: vec![],
                key: Some("c".into()),
            }),
        ];
        let new_children = vec![
            VDomNode::Element(ElementNode {
                tag: "div".into(),
                attrs: Default::default(),
                events: Default::default(),
                children: vec![],
                key: Some("c".into()),
            }),
            VDomNode::Element(ElementNode {
                tag: "div".into(),
                attrs: Default::default(),
                events: Default::default(),
                children: vec![],
                key: Some("a".into()),
            }),
            VDomNode::Element(ElementNode {
                tag: "div".into(),
                attrs: Default::default(),
                events: Default::default(),
                children: vec![],
                key: Some("b".into()),
            }),
        ];

        let old = VDomNode::Element(ElementNode {
            tag: "ul".into(),
            attrs: Default::default(),
            events: Default::default(),
            children: old_children,
            key: None,
        });
        let new = VDomNode::Element(ElementNode {
            tag: "ul".into(),
            attrs: Default::default(),
            events: Default::default(),
            children: new_children,
            key: None,
        });

        let result = diff(Some(&old), &new);
        let has_move = result.patches.iter().any(|p| matches!(p, Patch::Move { .. }));
        assert!(has_move, "Should detect move with keys");
    }

    #[test]
    fn test_attr_removed() {
        let old = VDomNode::element("div", vec![("class", "old"), ("id", "x")], vec![]);
        let new = VDomNode::element("div", vec![("class", "new")], vec![]);
        let result = diff(Some(&old), &new);
        let has_remove_attr = result
            .patches
            .iter()
            .any(|p| matches!(p, Patch::RemoveAttr { .. }));
        assert!(has_remove_attr);
    }

    #[test]
    fn test_text_unchanged() {
        let old = VDomNode::text("hello");
        let new = VDomNode::text("hello");
        let result = diff(Some(&old), &new);
        assert_eq!(result.patches.len(), 0);
    }

    #[test]
    fn test_element_attr_unchanged() {
        let old = VDomNode::element("div", vec![("class", "box")], vec![]);
        let new = VDomNode::element("div", vec![("class", "box")], vec![]);
        let result = diff(Some(&old), &new);
        assert_eq!(result.patches.len(), 0);
    }

    #[test]
    fn test_nested_element_diff() {
        let old = VDomNode::element(
            "div",
            vec![],
            vec![VDomNode::element("p", vec![("id", "a")], vec![VDomNode::text("old")])],
        );
        let new = VDomNode::element(
            "div",
            vec![],
            vec![VDomNode::element("p", vec![("id", "a")], vec![VDomNode::text("new")])],
        );
        let result = diff(Some(&old), &new);
        assert!(!result.patches.is_empty());
        let has_set_text = result.patches.iter().any(|p| matches!(p, Patch::SetText { .. }));
        assert!(has_set_text);
    }
}
