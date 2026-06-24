use crate::node::*;
use std::collections::HashMap;

pub enum HChild {
    Node(VDomNode),
    Text(String),
}

impl From<VDomNode> for HChild {
    fn from(node: VDomNode) -> Self {
        HChild::Node(node)
    }
}

impl From<&str> for HChild {
    fn from(s: &str) -> Self {
        HChild::Text(s.to_string())
    }
}

impl From<String> for HChild {
    fn from(s: String) -> Self {
        HChild::Text(s)
    }
}

pub fn h(
    tag: &str,
    attrs: Vec<(&str, &str)>,
    children: Vec<HChild>,
) -> VDomNode {
    let attr_map: HashMap<String, AttrValue> = attrs
        .into_iter()
        .map(|(k, v)| (k.to_string(), AttrValue::String(v.to_string())))
        .collect();

    let child_nodes: Vec<VDomNode> = children
        .into_iter()
        .map(|child| match child {
            HChild::Node(node) => node,
            HChild::Text(text) => VDomNode::text(&text),
        })
        .collect();

    VDomNode::Element(ElementNode {
        tag: tag.to_string(),
        attrs: attr_map,
        events: HashMap::new(),
        children: child_nodes,
        key: None,
    })
}

pub fn h_with_key(
    tag: &str,
    key: &str,
    attrs: Vec<(&str, &str)>,
    children: Vec<HChild>,
) -> VDomNode {
    let attr_map: HashMap<String, AttrValue> = attrs
        .into_iter()
        .map(|(k, v)| (k.to_string(), AttrValue::String(v.to_string())))
        .collect();

    let child_nodes: Vec<VDomNode> = children
        .into_iter()
        .map(|child| match child {
            HChild::Node(node) => node,
            HChild::Text(text) => VDomNode::text(&text),
        })
        .collect();

    VDomNode::Element(ElementNode {
        tag: tag.to_string(),
        attrs: attr_map,
        events: HashMap::new(),
        children: child_nodes,
        key: Some(key.to_string()),
    })
}

pub fn h_with_attrs(
    tag: &str,
    attrs: HashMap<String, AttrValue>,
    events: HashMap<EventType, u64>,
    children: Vec<VDomNode>,
    key: Option<String>,
) -> VDomNode {
    VDomNode::Element(ElementNode {
        tag: tag.to_string(),
        attrs,
        events,
        children,
        key,
    })
}

pub fn fragment(children: Vec<VDomNode>) -> VDomNode {
    VDomNode::fragment(children)
}

pub fn text(content: &str) -> VDomNode {
    VDomNode::text(content)
}
