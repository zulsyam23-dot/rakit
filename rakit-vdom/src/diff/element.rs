use crate::diff::*;
use crate::node::*;
use std::collections::HashMap;

impl Differ {
    pub fn diff_elements(
        &mut self,
        old: &ElementNode,
        new: &ElementNode,
        path: &[usize],
    ) {
        if old.tag != new.tag {
            self.stats.nodes_replaced += 1;
            self.patches.push(Patch::Replace {
                old_path: path.to_vec(),
                new_node: VDomNode::Element(new.clone()),
            });
            return;
        }

        self.diff_attrs(&old.attrs, &new.attrs, path);

        self.diff_events(&old.events, &new.events, path);

        self.diff_children(&old.children, &new.children, path, None);
    }

    pub fn diff_attrs(
        &mut self,
        old: &HashMap<String, AttrValue>,
        new: &HashMap<String, AttrValue>,
        path: &[usize],
    ) {
        for (name, _) in old.iter() {
            if !new.contains_key(name) {
                self.patches.push(Patch::RemoveAttr {
                    path: path.to_vec(),
                    name: name.clone(),
                });
                self.stats.attrs_changed += 1;
            }
        }

        for (name, new_val) in new.iter() {
            match old.get(name) {
                Some(old_val) if old_val != new_val => {
                    self.patches.push(Patch::SetAttr {
                        path: path.to_vec(),
                        name: name.clone(),
                        value: new_val.clone(),
                    });
                    self.stats.attrs_changed += 1;
                }
                None => {
                    self.patches.push(Patch::SetAttr {
                        path: path.to_vec(),
                        name: name.clone(),
                        value: new_val.clone(),
                    });
                    self.stats.attrs_changed += 1;
                }
                _ => {}
            }
        }
    }

    pub fn diff_events(
        &mut self,
        old: &HashMap<EventType, u64>,
        new: &HashMap<EventType, u64>,
        path: &[usize],
    ) {
        for (event_type, handler_id) in old.iter() {
            if !new.contains_key(event_type) {
                self.patches.push(Patch::DetachEvent {
                    path: path.to_vec(),
                    handler_id: *handler_id,
                });
                self.stats.events_changed += 1;
            }
        }

        for (event_type, new_handler) in new.iter() {
            match old.get(event_type) {
                Some(old_id) if old_id != new_handler => {
                    self.patches.push(Patch::DetachEvent {
                        path: path.to_vec(),
                        handler_id: *old_id,
                    });
                    self.patches.push(Patch::AttachEvent {
                        path: path.to_vec(),
                        event_type: event_type.clone(),
                        handler_id: *new_handler,
                    });
                    self.stats.events_changed += 1;
                }
                None => {
                    self.patches.push(Patch::AttachEvent {
                        path: path.to_vec(),
                        event_type: event_type.clone(),
                        handler_id: *new_handler,
                    });
                    self.stats.events_changed += 1;
                }
                _ => {}
            }
        }
    }
}
