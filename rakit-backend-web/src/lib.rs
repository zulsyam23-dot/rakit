pub mod cssom;
pub mod dom;
pub mod event;

use rakit_runtime::event::{dispatch_event, EventData, EventType};
use rakit_ui::backend::*;
use rakit_vdom::node::AttrValue;

pub struct WebBackend {
    dom: dom::DomBackend,
}

impl WebBackend {
    pub fn new(root_element_id: &str) -> Self {
        WebBackend {
            dom: dom::DomBackend::new(root_element_id),
        }
    }

    pub fn dom(&self) -> &dom::DomBackend {
        &self.dom
    }

    fn root_handle(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        if self.dom.get_node(0).is_none() {
            if let Some(node) = self.dom.root_node() {
                self.dom.cache_node(0, node);
            }
        }
        0
    }
}

impl UiBackend for WebBackend {
    type WindowHandle = u64;
    type ElementHandle = u64;
    type FontHandle = u64;

    fn init(&mut self, _config: &AppConfig) -> Result<()> {
        Ok(())
    }

    fn create_window(&mut self, _config: &WindowConfig) -> Result<u64> {
        Ok(self.dom.alloc_id())
    }

    fn root_element(&self, _window: &u64) -> u64 {
        self.root_handle()
    }

    fn run_event_loop(&mut self) -> Result<()> {
        Ok(())
    }

    fn quit(&mut self) {}

    fn resolve_path(&self, path: &[usize], root: &u64) -> Option<u64> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut current = self.dom.get_node(*root)?;
            for &idx in path {
                let children = current.child_nodes();
                current = children.get(idx as u32)?;
            }
            let id = self.dom.alloc_id();
            self.dom.cache_node(id, current);
            Some(id)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (path, root);
            None
        }
    }

    fn create_element(&mut self, _window: &u64, tag: &str) -> u64 {
        let id = self.dom.alloc_id();
        #[cfg(target_arch = "wasm32")]
        {
            let doc = web_sys::window()
                .and_then(|w| w.document())
                .expect("no document");
            let el = doc.create_element(tag).unwrap();
            self.dom.cache_node(id, el.into());
        }
        let _ = tag;
        id
    }

    fn create_text(&mut self, _window: &u64, text: &str) -> u64 {
        let id = self.dom.alloc_id();
        #[cfg(target_arch = "wasm32")]
        {
            let doc = web_sys::window()
                .and_then(|w| w.document())
                .expect("no document");
            let text_node = doc.create_text_node(text);
            self.dom.cache_node(id, text_node.into());
        }
        let _ = text;
        id
    }

    fn set_attribute(&mut self, elem: &u64, name: &str, value: &AttrValue) {
        let attr_str = match value {
            AttrValue::String(s) => s.clone(),
            AttrValue::Bool(b) => b.to_string(),
            AttrValue::Number(n) => n.to_string(),
            AttrValue::Expression(e) => e.clone(),
        };
        let dom_name = match name {
            "className" => "class",
            "htmlFor" => "for",
            other => other,
        };
        #[cfg(target_arch = "wasm32")]
        if let Some(node) = self.dom.get_node(*elem) {
            use wasm_bindgen::JsCast;
            if let Some(el) = node.dyn_ref::<web_sys::Element>() {
                let _ = el.set_attribute(dom_name, &attr_str);
            }
        }
        let _ = (elem, dom_name, attr_str);
    }

    fn remove_attribute(&mut self, elem: &u64, name: &str) {
        #[cfg(target_arch = "wasm32")]
        if let Some(node) = self.dom.get_node(*elem) {
            use wasm_bindgen::JsCast;
            if let Some(el) = node.dyn_ref::<web_sys::Element>() {
                let _ = el.remove_attribute(name);
            }
        }
        let _ = (elem, name);
    }

    fn set_text(&mut self, elem: &u64, text: &str) {
        #[cfg(target_arch = "wasm32")]
        if let Some(node) = self.dom.get_node(*elem) {
            node.set_text_content(Some(text));
        }
        let _ = (elem, text);
    }

    fn append_child(&mut self, parent: &u64, child: &u64) {
        #[cfg(target_arch = "wasm32")]
        if let (Some(parent_node), Some(child_node)) =
            (self.dom.get_node(*parent), self.dom.get_node(*child))
        {
            let _ = parent_node.append_child(&child_node);
        }
        let _ = (parent, child);
    }

    fn insert_child(&mut self, parent: &u64, child: &u64, index: usize) {
        #[cfg(target_arch = "wasm32")]
        if let (Some(parent_node), Some(child_node)) =
            (self.dom.get_node(*parent), self.dom.get_node(*child))
        {
            let child_count = parent_node.child_nodes().length();
            if index as u32 >= child_count {
                let _ = parent_node.append_child(&child_node);
            } else {
                let children = parent_node.child_nodes();
                if let Some(ref_node) = children.get(index as u32) {
                    let _ = parent_node.insert_before(&child_node, Some(&ref_node));
                } else {
                    let _ = parent_node.append_child(&child_node);
                }
            }
        }
        let _ = (parent, child, index);
    }

    fn remove_child(&mut self, parent: &u64, child: &u64) {
        #[cfg(target_arch = "wasm32")]
        if let (Some(parent_node), Some(child_node)) =
            (self.dom.get_node(*parent), self.dom.get_node(*child))
        {
            let _ = parent_node.remove_child(&child_node);
        }
        let _ = (parent, child);
        self.dom.remove_node(*child);
    }

    fn move_child(&mut self, parent: &u64, child: &u64, to_index: usize) {
        #[cfg(target_arch = "wasm32")]
        if let Some(child_node) = self.dom.get_node(*child) {
            if let Some(parent_node) = child_node.parent_node() {
                let _ = parent_node.remove_child(&child_node);
            }
            if let Some(new_parent) = self.dom.get_node(*parent) {
                let child_count = new_parent.child_nodes().length();
                if to_index as u32 >= child_count {
                    let _ = new_parent.append_child(&child_node);
                } else {
                    let children = new_parent.child_nodes();
                    if let Some(ref_node) = children.get(to_index as u32) {
                        let _ = new_parent.insert_before(&child_node, Some(&ref_node));
                    } else {
                        let _ = new_parent.append_child(&child_node);
                    }
                }
            }
        }
        let _ = (parent, child, to_index);
    }

    fn attach_event(&mut self, elem: &u64, event_type: EventType, handler_id: u64) {
        #[cfg(target_arch = "wasm32")]
        if let Some(node) = self.dom.get_node(*elem) {
            use wasm_bindgen::JsCast;
            let event_name = event_type.as_str().to_string();
            let h_id = handler_id;
            let closure = wasm_bindgen::prelude::Closure::wrap(
                Box::new(move |event: web_sys::Event| {
                    let data = dom::DomBackend::extract_wasm_event(&event);
                    rakit_runtime::event::dispatch_event(h_id, data);
                }) as Box<dyn FnMut(_)>,
            );
            if let Some(target) = node.dyn_ref::<web_sys::EventTarget>() {
                let _ = target.add_event_listener_with_callback(
                    &event_name,
                    closure.as_ref().unchecked_ref(),
                );
            }
            closure.forget();
        }
        let _ = (elem, event_type, handler_id);
    }

    fn detach_event(&mut self, elem: &u64, handler_id: u64) {
        let _ = (elem, handler_id);
    }

    fn dispatch_event(&self, handler_id: u64, data: EventData) {
        dispatch_event(handler_id, data);
    }

    fn apply_stylesheet(&mut self, _window: &u64, css: &str) {
        self.dom.inject_stylesheet(css);
    }

    fn set_style(&mut self, elem: &u64, property: &str, value: &str) {
        #[cfg(target_arch = "wasm32")]
        if let Some(node) = self.dom.get_node(*elem) {
            use wasm_bindgen::JsCast;
            if let Some(html_el) = node.dyn_ref::<web_sys::HtmlElement>() {
                let _ = html_el.style().set_property(property, value);
            }
        }
        let _ = (elem, property, value);
    }

    fn set_bounds(&mut self, _elem: &u64, _x: f64, _y: f64, _w: f64, _h: f64) {}

    fn measure(&mut self, _elem: &u64) -> (f64, f64) {
        (0.0, 0.0)
    }
}
