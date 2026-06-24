use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    String(String),
    Bool(bool),
    Number(f64),
    Expression(String),
}

impl AttrValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AttrValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl From<&str> for AttrValue {
    fn from(s: &str) -> Self {
        AttrValue::String(s.to_string())
    }
}

impl From<String> for AttrValue {
    fn from(s: String) -> Self {
        AttrValue::String(s)
    }
}

impl From<bool> for AttrValue {
    fn from(b: bool) -> Self {
        AttrValue::Bool(b)
    }
}

pub use rakit_runtime::event::EventType;

#[derive(Debug, Clone, PartialEq)]
pub struct ElementNode {
    pub tag: String,
    pub attrs: HashMap<String, AttrValue>,
    pub events: HashMap<EventType, u64>,
    pub children: Vec<VDomNode>,
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextNode {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentNode {
    pub children: Vec<VDomNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentNode {
    pub name: String,
    pub props: HashMap<String, AttrValue>,
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VDomNode {
    Element(ElementNode),
    Text(TextNode),
    Fragment(FragmentNode),
    Component(ComponentNode),
    Empty,
}

impl VDomNode {
    pub fn element(tag: &str, attrs: Vec<(&str, &str)>, children: Vec<VDomNode>) -> Self {
        let attr_map: HashMap<String, AttrValue> = attrs
            .into_iter()
            .map(|(k, v)| (k.to_string(), AttrValue::String(v.to_string())))
            .collect();
        VDomNode::Element(ElementNode {
            tag: tag.to_string(),
            attrs: attr_map,
            events: HashMap::new(),
            children,
            key: None,
        })
    }

    pub fn element_with_attrs(
        tag: &str,
        attrs: HashMap<String, AttrValue>,
        events: HashMap<EventType, u64>,
        children: Vec<VDomNode>,
        key: Option<String>,
    ) -> Self {
        VDomNode::Element(ElementNode {
            tag: tag.to_string(),
            attrs,
            events,
            children,
            key,
        })
    }

    pub fn text(value: &str) -> Self {
        VDomNode::Text(TextNode {
            value: value.to_string(),
        })
    }

    pub fn fragment(children: Vec<VDomNode>) -> Self {
        VDomNode::Fragment(FragmentNode { children })
    }

    pub fn component(name: &str, props: HashMap<String, AttrValue>, key: Option<String>) -> Self {
        VDomNode::Component(ComponentNode {
            name: name.to_string(),
            props,
            key,
        })
    }

    pub fn empty() -> Self {
        VDomNode::Empty
    }

    pub fn tag(&self) -> Option<&str> {
        match self {
            VDomNode::Element(e) => Some(&e.tag),
            _ => None,
        }
    }

    pub fn key(&self) -> Option<&str> {
        match self {
            VDomNode::Element(e) => e.key.as_deref(),
            VDomNode::Component(c) => c.key.as_deref(),
            _ => None,
        }
    }

    pub fn children(&self) -> &[VDomNode] {
        match self {
            VDomNode::Element(e) => &e.children,
            VDomNode::Fragment(f) => &f.children,
            _ => &[],
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<VDomNode>> {
        match self {
            VDomNode::Element(e) => Some(&mut e.children),
            VDomNode::Fragment(f) => Some(&mut f.children),
            _ => None,
        }
    }

    pub fn is_same_type(&self, other: &VDomNode) -> bool {
        match (self, other) {
            (VDomNode::Element(a), VDomNode::Element(b)) => a.tag == b.tag,
            (VDomNode::Text(_), VDomNode::Text(_)) => true,
            (VDomNode::Fragment(_), VDomNode::Fragment(_)) => true,
            (VDomNode::Component(a), VDomNode::Component(b)) => a.name == b.name,
            (VDomNode::Empty, VDomNode::Empty) => true,
            _ => false,
        }
    }

    pub fn flatten_fragments(self) -> Vec<VDomNode> {
        match self {
            VDomNode::Fragment(f) => f
                .children
                .into_iter()
                .flat_map(|c| c.flatten_fragments())
                .collect(),
            VDomNode::Element(e) => {
                let children: Vec<VDomNode> = e
                    .children
                    .into_iter()
                    .flat_map(|c| c.flatten_fragments())
                    .collect();
                vec![VDomNode::Element(ElementNode { children, ..e })]
            }
            other => vec![other],
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, VDomNode::Empty)
    }

    pub fn is_fragment(&self) -> bool {
        matches!(self, VDomNode::Fragment(_))
    }
}
