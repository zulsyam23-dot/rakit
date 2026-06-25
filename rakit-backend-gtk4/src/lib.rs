pub mod css;
pub mod event;
pub mod widget;
pub mod window;

use rakit_runtime::event::{dispatch_event, EventData, EventType};
use rakit_ui::backend::*;
use rakit_vdom::node::AttrValue;
use std::collections::HashMap;

pub struct Gtk4Backend {
    windows: HashMap<u64, GtkWindowData>,
    element_map: HashMap<u64, u64>,
    next_elem_id: u64,
}

#[allow(dead_code)]
struct GtkWindowData {
    window: window::GtkWindow,
}

impl Gtk4Backend {
    pub fn new() -> Self {
        Gtk4Backend {
            windows: HashMap::new(),
            element_map: HashMap::new(),
            next_elem_id: 1,
        }
    }

    fn init_gtk() {
        #[cfg(target_os = "linux")]
        {
            gtk4::init().ok();
        }
    }

    fn run_gtk_loop() {
        #[cfg(target_os = "linux")]
        {
            gtk4::main();
        }
    }

    fn quit_gtk_loop() {
        #[cfg(target_os = "linux")]
        {
            gtk4::main_quit();
        }
    }
}

impl UiBackend for Gtk4Backend {
    type WindowHandle = u64;
    type ElementHandle = u64;
    type FontHandle = u64;

    fn init(&mut self, _config: &AppConfig) -> Result<()> {
        Self::init_gtk();
        Ok(())
    }

    fn create_window(&mut self, config: &WindowConfig) -> Result<u64> {
        let id = self.next_elem_id;
        self.next_elem_id += 1;
        let win = window::GtkWindow::new(config, id);
        self.windows.insert(id, GtkWindowData { window: win });
        Ok(id)
    }

    fn root_element(&self, window: &u64) -> u64 {
        *window
    }

    fn run_event_loop(&mut self) -> Result<()> {
        Self::run_gtk_loop();
        Ok(())
    }

    fn quit(&mut self) {
        Self::quit_gtk_loop();
    }

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

    fn set_attribute(&mut self, _elem: &u64, _name: &str, _value: &AttrValue) {
        // GTK4 attribute setting handled by widget mapping
    }

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

    fn resolve_path(&self, _path: &[usize], _root: &u64) -> Option<u64> { None }

    fn dispatch_event(&self, handler_id: u64, data: EventData) {
        dispatch_event(handler_id, data);
    }

    fn apply_stylesheet(&mut self, _window: &u64, _css: &str) {}

    fn set_style(&mut self, _elem: &u64, _property: &str, _value: &str) {}

    fn set_bounds(&mut self, _elem: &u64, _x: f64, _y: f64, _w: f64, _h: f64) {}

    fn measure(&mut self, _elem: &u64) -> (f64, f64) {
        (0.0, 0.0)
    }
}
