use crate::diff::{DiffResult, Patch};
use crate::render::native::NativeRenderer;
use std::collections::HashMap;

pub type NativeHandle = u64;

pub struct NativePatchApplicator {
    renderer: NativeRenderer,
    handle_cache: HashMap<Vec<usize>, NativeHandle>,
    root_handle: NativeHandle,
    next_handle: NativeHandle,
}

impl NativePatchApplicator {
    pub fn new(renderer: NativeRenderer, root_handle: NativeHandle) -> Self {
        let mut cache = HashMap::new();
        cache.insert(vec![], root_handle);
        NativePatchApplicator {
            renderer,
            handle_cache: cache,
            root_handle,
            next_handle: root_handle + 1,
        }
    }

    pub fn apply_patches(&mut self, result: &DiffResult) {
        for patch in &result.patches {
            self.apply_patch(patch);
        }
    }

    fn apply_patch(&mut self, patch: &Patch) {
        match patch {
            Patch::Replace { old_path, new_node } => {
                if let Some(handle) = self.handle_cache.remove(old_path) {
                    let parent = self.get_parent_handle(old_path);
                    self.renderer.remove_child(&parent, &handle);
                }
                let new_handle = self.renderer.render_node(new_node, &self.root_handle);
                self.handle_cache.insert(old_path.clone(), new_handle);
            }

            Patch::Remove { path } => {
                if let Some(handle) = self.handle_cache.remove(path) {
                    let parent = self.get_parent_handle(path);
                    self.renderer.remove_child(&parent, &handle);
                }
            }

            Patch::Insert {
                parent_path,
                index,
                node,
            } => {
                let parent = if parent_path.is_empty() {
                    self.root_handle
                } else {
                    *self.handle_cache.get(parent_path).unwrap_or(&self.root_handle)
                };
                let handle = self.renderer.render_node(node, &parent);
                let mut child_path = parent_path.clone();
                child_path.push(*index);
                self.handle_cache.insert(child_path, handle);
            }

            Patch::Move {
                from_path,
                to_parent,
                to_index,
            } => {
                if let Some(handle) = self.handle_cache.remove(from_path) {
                    let old_parent = self.get_parent_handle(from_path);
                    self.renderer.remove_child(&old_parent, &handle);

                    let new_parent = if to_parent.is_empty() {
                        self.root_handle
                    } else {
                        *self.handle_cache.get(to_parent).unwrap_or(&self.root_handle)
                    };
                    self.renderer
                        .insert_child(&new_parent, &handle, *to_index);
                    let mut new_path = to_parent.clone();
                    new_path.push(*to_index);
                    self.handle_cache.insert(new_path, handle);
                }
            }

            Patch::SetAttr { path, name, value } => {
                if let Some(handle) = self.handle_cache.get(path) {
                    self.renderer.set_attribute(handle, name, value);
                }
            }

            Patch::RemoveAttr { path, name } => {
                if let Some(handle) = self.handle_cache.get(path) {
                    self.renderer.remove_attribute(handle, name);
                }
            }

            Patch::SetText { path, text } => {
                if let Some(handle) = self.handle_cache.get(path) {
                    self.renderer.set_text(handle, text);
                }
            }

            Patch::AttachEvent {
                path,
                event_type,
                handler_id,
            } => {
                if let Some(handle) = self.handle_cache.get(path) {
                    self.renderer
                        .attach_event(handle, event_type.clone(), *handler_id);
                }
            }

            Patch::DetachEvent { path, handler_id } => {
                if let Some(handle) = self.handle_cache.get(path) {
                    self.renderer.detach_event(handle, *handler_id);
                }
            }
        }
    }

    pub fn get_parent_handle(&self, path: &[usize]) -> NativeHandle {
        if path.len() <= 1 {
            self.root_handle
        } else {
            self.handle_cache
                .get(&path[..path.len() - 1].to_vec())
                .copied()
                .unwrap_or(self.root_handle)
        }
    }

    pub fn allocate_handle(&mut self) -> NativeHandle {
        let handle = self.next_handle;
        self.next_handle += 1;
        handle
    }
}
