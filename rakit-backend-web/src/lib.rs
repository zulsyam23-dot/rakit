pub mod cssom;
pub mod dom;
pub mod event;

use rakit_runtime::event::{dispatch_event, EventData, EventType};
use rakit_ui::backend::*;
use rakit_vdom::node::AttrValue;
use std::collections::HashMap;

pub struct WebBackend {
    dom: dom::DomBackend,
    element_map: HashMap<u64, u64>,
    next_elem_id: u64,
}

impl WebBackend {
    pub fn new(root_element_id: &str) -> Self {
        WebBackend {
            dom: dom::DomBackend::new(root_element_id),
            element_map: HashMap::new(),
            next_elem_id: 1,
        }
    }

    pub fn dom(&self) -> &dom::DomBackend {
        &self.dom
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
        let id = self.next_elem_id;
        self.next_elem_id += 1;
        self.element_map.insert(id, id);
        Ok(id)
    }

    fn root_element(&self, _window: &u64) -> u64 {
        // For web, the root is the element with root_element_id
        0
    }

    fn run_event_loop(&mut self) -> Result<()> {
        // WASM event loop is browser-driven
        Ok(())
    }

    fn quit(&mut self) {}

    fn create_element(&mut self, _window: &u64, _tag: &str) -> u64 {
        let id = self.next_elem_id;
        self.next_elem_id += 1;
        self.element_map.insert(id, id);
        id
    }

    fn create_text(&mut self, _window: &u64, _text: &str) -> u64 {
        let id = self.next_elem_id;
        self.next_elem_id += 1;
        self.element_map.insert(id, id);
        id
    }

    fn set_attribute(&mut self, _elem: &u64, _name: &str, _value: &AttrValue) {}

    fn remove_attribute(&mut self, _elem: &u64, _name: &str) {}

    fn set_text(&mut self, _elem: &u64, _text: &str) {}

    fn append_child(&mut self, _parent: &u64, _child: &u64) {}

    fn insert_child(&mut self, _parent: &u64, _child: &u64, _index: usize) {}

    fn remove_child(&mut self, _parent: &u64, child: &u64) {
        self.element_map.remove(child);
    }

    fn move_child(&mut self, _parent: &u64, _child: &u64, _to_index: usize) {}

    fn attach_event(&mut self, _elem: &u64, _event_type: EventType, _handler_id: u64) {}

    fn detach_event(&mut self, _elem: &u64, _handler_id: u64) {}

    fn dispatch_event(&self, handler_id: u64, data: EventData) {
        dispatch_event(handler_id, data);
    }

    fn apply_stylesheet(&mut self, _window: &u64, css: &str) {
        self.dom.inject_stylesheet(css);
    }

    fn set_style(&mut self, _elem: &u64, _property: &str, _value: &str) {}

    fn set_bounds(&mut self, _elem: &u64, _x: f64, _y: f64, _w: f64, _h: f64) {}

    fn measure(&mut self, _elem: &u64) -> (f64, f64) {
        (0.0, 0.0)
    }
}
