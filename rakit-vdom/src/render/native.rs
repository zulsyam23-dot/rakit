use crate::node::*;

pub struct NativeRenderer;

impl NativeRenderer {
    pub fn new() -> Self {
        NativeRenderer
    }

    pub fn render_node(&mut self, node: &VDomNode, _parent_handle: &u64) -> u64 {
        match node {
            VDomNode::Element(elem) => {
                let handle = allocate_id();
                for (name, value) in &elem.attrs {
                    self.set_attribute(&handle, name, value);
                }
                for child in &elem.children {
                    let child_handle = self.render_node(child, &handle);
                    self.append_child(&handle, &child_handle);
                }
                handle
            }
            VDomNode::Text(_text) => {
                let handle = allocate_id();
                handle
            }
            VDomNode::Fragment(frag) => {
                let mut last = 0;
                for child in &frag.children {
                    last = self.render_node(child, _parent_handle);
                }
                last
            }
            VDomNode::Component(_comp) => {
                let handle = allocate_id();
                handle
            }
            VDomNode::Empty => 0,
        }
    }

    pub fn remove_child(&self, _parent: &u64, _child: &u64) {}

    pub fn insert_child(&self, _parent: &u64, _child: &u64, _index: usize) {}

    pub fn append_child(&self, _parent: &u64, _child: &u64) {}

    pub fn set_attribute(&self, _handle: &u64, _name: &str, _value: &AttrValue) {}

    pub fn remove_attribute(&self, _handle: &u64, _name: &str) {}

    pub fn set_text(&self, _handle: &u64, _text: &str) {}

    pub fn attach_event(&self, _handle: &u64, _event_type: EventType, _handler_id: u64) {}

    pub fn detach_event(&self, _handle: &u64, _handler_id: u64) {}
}

static mut NEXT_ID: u64 = 1;

fn allocate_id() -> u64 {
    unsafe {
        let id = NEXT_ID;
        NEXT_ID += 1;
        id
    }
}
