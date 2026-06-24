use crate::diff::*;
use crate::node::*;
use crate::utils;
use std::collections::HashMap;

impl Differ {
    pub fn diff_children(
        &mut self,
        old: &[VDomNode],
        new: &[VDomNode],
        parent_path: &[usize],
        _parent_key: Option<&str>,
    ) {
        let mut old_key_map: HashMap<&str, (usize, &VDomNode)> = HashMap::new();
        for (i, child) in old.iter().enumerate() {
            if let Some(key) = utils::get_node_key(child) {
                old_key_map.insert(key, (i, child));
            }
        }

        let mut old_used = vec![false; old.len()];
        let mut last_placed_index = 0;
        let mut _reorder_buffer: Vec<Option<&VDomNode>> = Vec::new();

        for (new_idx, new_child) in new.iter().enumerate() {
            let key = utils::get_node_key(new_child);

            if let Some(key) = key {
                if let Some(&(old_idx, _)) = old_key_map.get(key) {
                    old_used[old_idx] = true;

                    if old_idx < last_placed_index {
                        let mut from_path = parent_path.to_vec();
                        from_path.push(old_idx);
                        self.patches.push(Patch::Move {
                            from_path,
                            to_parent: parent_path.to_vec(),
                            to_index: new_idx,
                        });
                        self.stats.nodes_moved += 1;
                    }
                    last_placed_index = std::cmp::max(last_placed_index, old_idx);

                    let mut child_path = parent_path.to_vec();
                    child_path.push(new_idx);
                    self.diff_node(Some(&old[old_idx]), new_child, &mut child_path);
                } else {
                    let mut child_path = parent_path.to_vec();
                    child_path.push(new_idx);
                    self.patches.push(Patch::Insert {
                        parent_path: parent_path.to_vec(),
                        index: new_idx,
                        node: new_child.clone(),
                    });
                }
            } else {
                let mut child_path = parent_path.to_vec();
                child_path.push(new_idx);
                let old_child = old.get(new_idx);
                self.diff_node(old_child, new_child, &mut child_path);
                if new_idx < old.len() {
                    old_used[new_idx] = true;
                }
            }
        }

        for (old_idx, used) in old_used.iter().enumerate() {
            if !used {
                let mut path = parent_path.to_vec();
                path.push(old_idx);
                self.patches.push(Patch::Remove { path });
            }
        }
    }

    pub fn diff_components(
        &mut self,
        old: &ComponentNode,
        new: &ComponentNode,
        path: &[usize],
    ) {
        if old.name != new.name {
            self.patches.push(Patch::Replace {
                old_path: path.to_vec(),
                new_node: VDomNode::Component(new.clone()),
            });
            return;
        }

        if utils::props_changed(&old.props, &new.props) {
            self.patches.push(Patch::Replace {
                old_path: path.to_vec(),
                new_node: VDomNode::Component(new.clone()),
            });
        }
    }
}
