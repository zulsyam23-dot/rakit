use crate::node::*;
use std::collections::HashMap;

pub fn get_node_key(node: &VDomNode) -> Option<&str> {
    match node {
        VDomNode::Element(e) => e.key.as_deref(),
        VDomNode::Component(c) => c.key.as_deref(),
        _ => None,
    }
}

pub fn props_changed(
    old: &HashMap<String, AttrValue>,
    new: &HashMap<String, AttrValue>,
) -> bool {
    if old.len() != new.len() {
        return true;
    }
    for (key, new_val) in new {
        match old.get(key) {
            Some(old_val) if old_val == new_val => continue,
            _ => return true,
        }
    }
    false
}

pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
