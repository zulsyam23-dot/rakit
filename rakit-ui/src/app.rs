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
                    let mut rendered = factory(&comp.props);
                    if !comp.children.is_empty() {
                        if let Some(children_mut) = rendered.children_mut() {
                            *children_mut = comp.children.clone();
                        }
                    }
                    Self::render_to_native(backend, &rendered, parent, window, registry);
                } else if !comp.children.is_empty() {
                    for child in &comp.children {
                        Self::render_to_native(backend, child, parent, window, registry);
                    }
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
                if let Some(window) = self.windows.first().cloned() {
                    let root_handle = self.backend.root_element(&window);
                    self.apply_patches(&result.patches, &root_handle, &window);
                }
            }
        }

        self.last_vdom = Some(new_vdom);
    }

    fn apply_patches(
        &mut self,
        patches: &[rakit_vdom::diff::Patch],
        root: &B::ElementHandle,
        window: &B::WindowHandle,
    ) {
        for patch in patches {
            match patch {
                rakit_vdom::diff::Patch::SetAttr {
                    path,
                    name,
                    value,
                } => {
                    if let Some(handle) = self.backend.resolve_path(path, root) {
                        self.backend.set_attribute(&handle, name, value);
                    }
                }
                rakit_vdom::diff::Patch::SetText { path, text } => {
                    if let Some(handle) = self.backend.resolve_path(path, root) {
                        self.backend.set_text(&handle, text);
                    }
                }
                rakit_vdom::diff::Patch::AttachEvent {
                    path,
                    event_type,
                    handler_id,
                } => {
                    if let Some(handle) = self.backend.resolve_path(path, root) {
                        self.backend.attach_event(&handle, event_type.clone(), *handler_id);
                    }
                }
                rakit_vdom::diff::Patch::DetachEvent { path, handler_id } => {
                    if let Some(handle) = self.backend.resolve_path(path, root) {
                        self.backend.detach_event(&handle, *handler_id);
                    }
                }
                rakit_vdom::diff::Patch::Remove { path } => {
                    if let Some(handle) = self.backend.resolve_path(path, root) {
                        if path.len() >= 1 {
                            let parent_path = path[..path.len() - 1].to_vec();
                            if let Some(parent) = self.backend.resolve_path(&parent_path, root) {
                                self.backend.remove_child(&parent, &handle);
                            }
                        }
                    }
                }
                rakit_vdom::diff::Patch::Insert {
                    parent_path,
                    index,
                    node,
                } => {
                    if let Some(parent) = self.backend.resolve_path(parent_path, root) {
                        let child = self.render_node_wasm(node, window);
                        self.backend.insert_child(&parent, &child, *index);
                    }
                }
                rakit_vdom::diff::Patch::Replace { old_path, new_node } => {
                    if let Some(old_handle) = self.backend.resolve_path(old_path, root) {
                        if let Some(parent_node) = old_path
                            .len()
                            .checked_sub(1)
                            .and_then(|i| self.backend.resolve_path(&old_path[..i].to_vec(), root))
                        {
                            self.backend.remove_child(&parent_node, &old_handle);
                            let new_child = self.render_node_wasm(new_node, window);
                            self.backend.append_child(&parent_node, &new_child);
                        }
                    }
                }
                rakit_vdom::diff::Patch::Move {
                    from_path,
                    to_parent,
                    to_index,
                } => {
                    if let Some(handle) = self.backend.resolve_path(from_path, root) {
                        if let Some(from_parent) = from_path
                            .len()
                            .checked_sub(1)
                            .and_then(|i| self.backend.resolve_path(&from_path[..i].to_vec(), root))
                        {
                            self.backend.remove_child(&from_parent, &handle);
                            if let Some(dest) = self.backend.resolve_path(to_parent, root) {
                                self.backend.insert_child(&dest, &handle, *to_index);
                            }
                        }
                    }
                }
                rakit_vdom::diff::Patch::RemoveAttr { path, name } => {
                    if let Some(handle) = self.backend.resolve_path(path, root) {
                        self.backend.remove_attribute(&handle, name);
                    }
                }
            }
        }
    }

    fn render_node_wasm(&mut self, node: &VDomNode, window: &B::WindowHandle) -> B::ElementHandle {
        match node {
            VDomNode::Element(elem) => {
                let handle = self.backend.create_element(window, &elem.tag);
                for (name, value) in &elem.attrs {
                    self.backend.set_attribute(&handle, name, value);
                }
                for child in &elem.children {
                    let child_handle = self.render_node_wasm(child, window);
                    self.backend.append_child(&handle, &child_handle);
                }
                handle
            }
            VDomNode::Text(text) => self.backend.create_text(window, &text.value),
            VDomNode::Fragment(frag) => {
                let first_child = frag.children.first().map(|c| self.render_node_wasm(c, window));
                for child in &frag.children[1..] {
                    let _ = self.render_node_wasm(child, window);
                }
                first_child.unwrap_or_else(|| self.backend.create_text(window, ""))
            }
            VDomNode::Component(comp) => {
                if let Some(factory) = self.component_registry.get(&comp.name) {
                    let rendered = factory(&comp.props);
                    self.render_node_wasm(&rendered, window)
                } else {
                    self.backend.create_text(window, "")
                }
            }
            VDomNode::Empty => self.backend.create_text(window, ""),
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
