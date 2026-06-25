use rakit_vdom::diff::Patch;
use rakit_vdom::node::{AttrValue, VDomNode};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
type NodeCache = web_sys::Node;

#[cfg(not(target_arch = "wasm32"))]
type NodeCache = u64;

#[allow(dead_code)]
pub struct DomBackend {
    root_id: String,
    node_cache: RefCell<HashMap<u64, NodeCache>>,
    next_id: Cell<u64>,
    styles: RefCell<Vec<String>>,
}

#[cfg(target_arch = "wasm32")]
fn document() -> web_sys::Document {
    web_sys::window()
        .and_then(|w| w.document())
        .expect("no document")
}

impl DomBackend {
    pub fn new(root_id: &str) -> Self {
        DomBackend {
            root_id: root_id.to_string(),
            node_cache: RefCell::new(HashMap::new()),
            next_id: Cell::new(1),
            styles: RefCell::new(Vec::new()),
        }
    }

    pub fn alloc_id(&self) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    #[cfg(target_arch = "wasm32")]
    pub fn cache_node(&self, id: u64, node: web_sys::Node) {
        self.node_cache.borrow_mut().insert(id, node);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn cache_node(&self, id: u64, node: u64) {
        self.node_cache.borrow_mut().insert(id, node);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_node(&self, id: u64) -> Option<web_sys::Node> {
        self.node_cache.borrow().get(&id).cloned()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_node(&self, id: u64) -> Option<u64> {
        self.node_cache.borrow().get(&id).copied()
    }

    pub fn remove_node(&self, id: u64) {
        self.node_cache.borrow_mut().remove(&id);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn root_node(&self) -> Option<web_sys::Node> {
        let doc = document();
        if let Some(el) = doc.get_element_by_id(&self.root_id) {
            return Some(el.into());
        }
        if let Some(el) = doc.get_element_by_id("root") {
            return Some(el.into());
        }
        doc.body().map(|b| b.into())
    }

    pub fn render_to_dom(&self, node: &VDomNode, parent_id: u64) {
        #[cfg(target_arch = "wasm32")]
        self.render_to_dom_wasm(node, parent_id);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (node, parent_id);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn render_to_dom_wasm(&self, node: &VDomNode, parent_id: u64) {
        use wasm_bindgen::JsCast;

        match node {
            VDomNode::Element(elem) => {
                let doc = document();
                let el: web_sys::Element = doc.create_element(&elem.tag).unwrap();

                for (name, value) in &elem.attrs {
                    let _ = el.set_attribute(name, &self.attr_str(value));
                }

                let child_id = self.alloc_id();
                let node_ref: web_sys::Node = el.unchecked_into();
                self.node_cache.borrow_mut().insert(child_id, node_ref.clone());

                for child in &elem.children {
                    self.render_to_dom_wasm(child, child_id);
                }

                if let Some(parent) = self.node_cache.borrow().get(&parent_id) {
                    let _ = parent.append_child(&node_ref);
                }
            }
            VDomNode::Text(text) => {
                let doc = document();
                let text_node = doc.create_text_node(&text.value);
                if let Some(parent) = self.node_cache.borrow().get(&parent_id) {
                    let _ = parent.append_child(&text_node);
                }
            }
            VDomNode::Fragment(frag) => {
                for child in &frag.children {
                    self.render_to_dom_wasm(child, parent_id);
                }
            }
            VDomNode::Empty => {}
            VDomNode::Component(_) => {}
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn create_dom_node(&self, node: &VDomNode) -> (u64, web_sys::Node) {
        use wasm_bindgen::JsCast;

        match node {
            VDomNode::Element(elem) => {
                let doc = document();
                let el: web_sys::Element = doc.create_element(&elem.tag).unwrap();

                for (name, value) in &elem.attrs {
                    let _ = el.set_attribute(name, &self.attr_str(value));
                }

                let child_id = self.alloc_id();
                let node_ref: web_sys::Node = el.unchecked_into();
                self.node_cache.borrow_mut().insert(child_id, node_ref.clone());

                for child in &elem.children {
                    self.render_to_dom_wasm(child, child_id);
                }

                (child_id, node_ref)
            }
            VDomNode::Text(text) => {
                let doc = document();
                let text_node = doc.create_text_node(&text.value);
                let id = self.alloc_id();
                (id, text_node.into())
            }
            VDomNode::Fragment(frag) => {
                let doc = document();
                let frag_node = doc.create_document_fragment();
                for child in &frag.children {
                    let (_, child_node) = self.create_dom_node(child);
                    let _ = frag_node.append_child(&child_node);
                }
                let id = self.alloc_id();
                self.node_cache.borrow_mut().insert(id, frag_node.clone().into());
                (id, frag_node.into())
            }
            VDomNode::Empty => {
                let doc = document();
                let text_node = doc.create_text_node("");
                let id = self.alloc_id();
                (id, text_node.into())
            }
            VDomNode::Component(comp) => {
                let doc = document();
                let el: web_sys::Element = doc.create_element("div").unwrap();
                el.set_attribute("data-component", &comp.name).unwrap();
                let id = self.alloc_id();
                let node_ref: web_sys::Node = el.unchecked_into();
                self.node_cache.borrow_mut().insert(id, node_ref.clone());
                (id, node_ref)
            }
        }
    }

    pub fn apply_dom_patches(&self, patches: &[Patch]) {
        #[cfg(target_arch = "wasm32")]
        self.apply_dom_patches_wasm(patches);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = patches;
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn apply_dom_patches_wasm(&self, patches: &[Patch]) {
        use wasm_bindgen::JsCast;

        for patch in patches {
            match patch {
                Patch::SetAttr { path, name, value } => {
                    if let Some(node) = self.resolve_path(path) {
                        if let Some(el) = node.dyn_ref::<web_sys::Element>() {
                            let _ = el.set_attribute(name, &self.attr_str(value));
                        }
                    }
                }
                Patch::SetText { path, text } => {
                    if let Some(node) = self.resolve_path(path) {
                        node.set_text_content(Some(text));
                    }
                }
                Patch::AttachEvent {
                    path,
                    event_type,
                    handler_id,
                } => {
                    if let Some(node) = self.resolve_path(path) {
                        let event_name = event_type.as_str().to_string();
                        let h_id = *handler_id;
                        let closure = wasm_bindgen::prelude::Closure::wrap(
                            Box::new(move |event: web_sys::Event| {
                                let data = Self::extract_wasm_event(&event);
                                rakit_runtime::event::dispatch_event(h_id, data);
                            }) as Box<dyn FnMut(_)>,
                        );
                        if let Some(target) = node.dyn_ref::<web_sys::EventTarget>() {
                            let _ = target
                                .add_event_listener_with_callback(
                                    &event_name,
                                    closure.as_ref().unchecked_ref(),
                                );
                        }
                        closure.forget();
                    }
                }
                Patch::DetachEvent { path, handler_id } => {
                    if let Some(node) = self.resolve_path(path) {
                        let event_name = handler_id.to_string();
                        if let Some(el) = node.dyn_ref::<web_sys::Element>() {
                            let _ = el.remove_attribute(&format!("data-on-{}", event_name));
                        }
                        let _ = handler_id;
                    }
                }
                Patch::Remove { path } => {
                    if let Some(node) = self.resolve_path(path) {
                        if let Some(parent) = node.parent_node() {
                            let _ = parent.remove_child(&node);
                        }
                    }
                }
                Patch::Insert {
                    parent_path,
                    index,
                    node,
                } => {
                    if let Some(parent) = self.resolve_path(parent_path) {
                        let (_, new_node) = self.create_dom_node(node);
                        let child_count = parent.child_nodes().length();
                        if *index as u32 >= child_count {
                            let _ = parent.append_child(&new_node);
                        } else {
                            let children = parent.child_nodes();
                            if let Some(ref_node) = children.get(*index as u32) {
                                let _ = parent
                                    .insert_before(&new_node, Some(&ref_node));
                            } else {
                                let _ = parent.append_child(&new_node);
                            }
                        }
                        let _ = index;
                    }
                }
                Patch::Replace { old_path, new_node } => {
                    if let Some(old) = self.resolve_path(old_path) {
                        if let Some(parent) = old.parent_node() {
                            let (_, fresh_node) = self.create_dom_node(new_node);
                            let _ = parent.replace_child(&fresh_node, &old);
                        }
                    }
                }
                Patch::Move {
                    from_path,
                    to_parent,
                    to_index,
                } => {
                    if let Some(node) = self.resolve_path(from_path) {
                        if let Some(parent) = node.parent_node() {
                            let _ = parent.remove_child(&node);
                        }
                        if let Some(new_parent) = self.resolve_path(to_parent) {
                            let child_count = new_parent.child_nodes().length();
                            if *to_index as u32 >= child_count {
                                let _ = new_parent.append_child(&node);
                            } else {
                                let children = new_parent.child_nodes();
                                if let Some(ref_node) = children.get(*to_index as u32) {
                                    let _ = new_parent
                                        .insert_before(&node, Some(&ref_node));
                                } else {
                                    let _ = new_parent.append_child(&node);
                                }
                            }
                        }
                    }
                }
                Patch::RemoveAttr { path, name } => {
                    if let Some(node) = self.resolve_path(path) {
                        if let Some(el) = node.dyn_ref::<web_sys::Element>() {
                            let _ = el.remove_attribute(name);
                        }
                    }
                }
            }
        }
    }

    fn attr_str(&self, value: &AttrValue) -> String {
        match value {
            AttrValue::String(s) => s.clone(),
            AttrValue::Bool(b) => b.to_string(),
            AttrValue::Number(n) => n.to_string(),
            AttrValue::Expression(e) => e.clone(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn resolve_path(&self, path: &[usize]) -> Option<web_sys::Node> {
        use wasm_bindgen::JsCast;

        let first_id = self.node_cache.borrow().keys().next().copied()?;
        let mut current = self.node_cache.borrow().get(&first_id)?.clone();
        for &idx in path {
            let children = current.child_nodes();
            current = children.get(idx as u32)?;
        }
        Some(current)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn extract_wasm_event(event: &web_sys::Event) -> rakit_runtime::event::EventData {
        use wasm_bindgen::JsCast;

        let event_type = rakit_runtime::event::EventType::from(event.type_().as_str());
        let mut data = rakit_runtime::event::EventData::new(event_type);

        data.target_id = event
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|e| e.get_attribute("id"));

        if let Some(ke) = event.dyn_ref::<web_sys::KeyboardEvent>() {
            data.key = Some(ke.key());
        }

        if let Some(me) = event.dyn_ref::<web_sys::MouseEvent>() {
            data.mouse_x = Some(me.client_x() as f64);
            data.mouse_y = Some(me.client_y() as f64);
        }

        if let Some(target) = event.target() {
            if let Some(input) = target.dyn_into::<web_sys::HtmlInputElement>().ok() {
                data.value = Some(input.value());
            }
        }

        data
    }

    pub fn inject_stylesheet(&self, css: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            let doc = document();
            if let Some(head) = doc.head() {
                let style = doc.create_element("style").unwrap();
                style.set_text_content(Some(css));
                let _ = head.append_child(&style);
            }
        }
        let _ = css;
        self.styles.borrow_mut().push(css.to_string());
    }
}
