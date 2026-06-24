use rakit_runtime::event::EventData;
use rakit_runtime::scheduler::Scheduler;
use rakit_vdom::diff::diff;
use rakit_vdom::node::{AttrValue, VDomNode};
use std::collections::HashMap;

use super::backend::*;

pub type ComponentFactory = Box<dyn Fn(&HashMap<String, AttrValue>) -> VDomNode>;

pub struct RakitApp<B: UiBackend> {
    pub backend: B,
    pub root_component: String,
    pub windows: Vec<B::WindowHandle>,
    pub component_registry: HashMap<String, ComponentFactory>,
    pub scheduler: Option<Scheduler>,
    pub last_vdom: Option<VDomNode>,
}

impl<B: UiBackend> RakitApp<B> {
    pub fn new(backend: B, root_component: &str) -> Self {
        RakitApp {
            backend,
            root_component: root_component.to_string(),
            windows: Vec::new(),
            component_registry: HashMap::new(),
            scheduler: None,
            last_vdom: None,
        }
    }

    pub fn register_component<F>(&mut self, name: &str, factory: F)
    where
        F: Fn(&HashMap<String, AttrValue>) -> VDomNode + 'static,
    {
        self.component_registry
            .insert(name.to_string(), Box::new(factory));
    }

    pub fn init(&mut self, config: &AppConfig) -> Result<()> {
        self.scheduler = Some(Scheduler::new());
        self.backend.init(config)
    }

    pub fn mount(&mut self, window_config: &WindowConfig) -> Result<()> {
        let window = self.backend.create_window(window_config)?;

        let vdom = self.render_root_component();
        let root_handle = self.backend.root_element(&window);
        Self::render_to_native(&mut self.backend, &vdom, &root_handle, &window, &self.component_registry);
        self.last_vdom = Some(vdom);

        self.windows.push(window);
        Ok(())
    }

    fn render_root_component(&self) -> VDomNode {
        if let Some(factory) = self.component_registry.get(&self.root_component) {
            let empty_props = HashMap::new();
            factory(&empty_props)
        } else {
            VDomNode::text("Root component not found")
        }
    }

    fn render_to_native(
        backend: &mut B,
        node: &VDomNode,
        parent: &B::ElementHandle,
        window: &B::WindowHandle,
        registry: &HashMap<String, ComponentFactory>,
    ) {
        match node {
            VDomNode::Element(elem) => {
                let handle = backend.create_element(window, &elem.tag);
                for (name, value) in &elem.attrs {
                    backend.set_attribute(&handle, name, value);
                }
                for (event_type, handler_id) in &elem.events {
                    backend.attach_event(&handle, event_type.clone(), *handler_id);
                }
                for child in &elem.children {
                    Self::render_to_native(backend, child, &handle, window, registry);
                }
                backend.append_child(parent, &handle);
            }
            VDomNode::Text(text) => {
                let handle = backend.create_text(window, &text.value);
                backend.append_child(parent, &handle);
            }
            VDomNode::Fragment(frag) => {
                for child in &frag.children {
                    Self::render_to_native(backend, child, parent, window, registry);
                }
            }
            VDomNode::Component(comp) => {
                if let Some(factory) = registry.get(&comp.name) {
                    let rendered = factory(&comp.props);
                    Self::render_to_native(backend, &rendered, parent, window, registry);
                }
            }
            VDomNode::Empty => {}
        }
    }

    pub fn update(&mut self) {
        let new_vdom = self.render_root_component();
        let old_vdom = self.last_vdom.take();

        if let Some(ref old) = old_vdom {
            let result = diff(Some(old), &new_vdom);

            if !result.patches.is_empty() {
                if let Some(window) = self.windows.first() {
                    let root_handle = self.backend.root_element(window);
                    self.apply_patches(&result.patches, &root_handle, window);
                }
            }
        }

        self.last_vdom = Some(new_vdom);
    }

    fn apply_patches(
        &self,
        patches: &[rakit_vdom::diff::Patch],
        _root: &B::ElementHandle,
        _window: &B::WindowHandle,
    ) {
        for patch in patches {
            match patch {
                rakit_vdom::diff::Patch::SetAttr {
                    path: _path,
                    name,
                    value,
                } => {
                    // TODO: resolve path to element handle and apply
                    let _ = (name, value);
                }
                rakit_vdom::diff::Patch::SetText { path: _path, text } => {
                    let _ = text;
                }
                rakit_vdom::diff::Patch::AttachEvent {
                    path: _path,
                    event_type,
                    handler_id,
                } => {
                    let _ = (event_type, handler_id);
                }
                rakit_vdom::diff::Patch::DetachEvent {
                    path: _path,
                    handler_id,
                } => {
                    let _ = handler_id;
                }
                rakit_vdom::diff::Patch::Remove { path: _path } => {}
                rakit_vdom::diff::Patch::Insert {
                    parent_path: _parent_path,
                    index: _index,
                    node: _node,
                } => {}
                rakit_vdom::diff::Patch::Replace {
                    old_path: _old_path,
                    new_node: _new_node,
                } => {}
                rakit_vdom::diff::Patch::Move {
                    from_path: _from_path,
                    to_parent: _to_parent,
                    to_index: _to_index,
                } => {}
                rakit_vdom::diff::Patch::RemoveAttr {
                    path: _path,
                    name: _name,
                } => {}
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.backend.run_event_loop()
    }

    pub fn quit(&mut self) {
        self.backend.quit();
    }

    pub fn dispatch_event(&self, handler_id: u64, data: EventData) {
        self.backend.dispatch_event(handler_id, data);
    }
}
