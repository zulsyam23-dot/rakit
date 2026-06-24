use std::collections::{HashMap, HashSet};

pub struct ScopeTree {
    parent_map: HashMap<u64, Option<u64>>,
    children_map: HashMap<u64, Vec<u64>>,
}

impl ScopeTree {
    pub fn new() -> Self {
        ScopeTree {
            parent_map: HashMap::new(),
            children_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, fiber_id: u64, parent: Option<u64>) {
        self.parent_map.insert(fiber_id, parent);
        if let Some(pid) = parent {
            self.children_map
                .entry(pid)
                .or_insert_with(Vec::new)
                .push(fiber_id);
        }
    }

    pub fn unregister(&mut self, fiber_id: u64) {
        if let Some(parent) = self.parent_map.remove(&fiber_id) {
            if let Some(pid) = parent {
                if let Some(children) = self.children_map.get_mut(&pid) {
                    children.retain(|c| *c != fiber_id);
                }
            }
        }
        self.children_map.remove(&fiber_id);
    }

    pub fn get_parent(&self, fiber_id: u64) -> Option<Option<u64>> {
        self.parent_map.get(&fiber_id).copied()
    }

    pub fn get_children(&self, fiber_id: u64) -> Vec<u64> {
        self.children_map
            .get(&fiber_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn ancestors(&self, fiber_id: u64) -> Vec<u64> {
        let mut result = Vec::new();
        let mut current = Some(fiber_id);
        while let Some(fid) = current {
            result.push(fid);
            current = self.parent_map.get(&fid).copied().flatten();
        }
        result
    }

    pub fn descendants(&self, fiber_id: u64) -> Vec<u64> {
        let mut result = Vec::new();
        let mut stack = vec![fiber_id];
        while let Some(fid) = stack.pop() {
            if let Some(children) = self.children_map.get(&fid) {
                for child in children {
                    result.push(*child);
                    stack.push(*child);
                }
            }
        }
        result
    }

    pub fn is_descendant_of(&self, fiber_id: u64, ancestor: u64) -> bool {
        let mut current = self.parent_map.get(&fiber_id).copied().flatten();
        while let Some(pid) = current {
            if pid == ancestor {
                return true;
            }
            current = self.parent_map.get(&pid).copied().flatten();
        }
        false
    }

    pub fn affected_by_provider(&self, provider_fiber: u64) -> HashSet<u64> {
        let mut affected = HashSet::new();
        affected.insert(provider_fiber);
        for desc in self.descendants(provider_fiber) {
            affected.insert(desc);
        }
        affected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_ancestors() {
        let mut tree = ScopeTree::new();
        tree.register(1, None);
        tree.register(2, Some(1));
        tree.register(3, Some(2));
        tree.register(4, Some(1));

        let anc = tree.ancestors(3);
        assert!(anc.contains(&3));
        assert!(anc.contains(&2));
        assert!(anc.contains(&1));
    }

    #[test]
    fn test_scope_descendants() {
        let mut tree = ScopeTree::new();
        tree.register(1, None);
        tree.register(2, Some(1));
        tree.register(3, Some(2));
        tree.register(4, Some(1));

        let desc = tree.descendants(1);
        assert_eq!(desc.len(), 3);
    }

    #[test]
    fn test_is_descendant_of() {
        let mut tree = ScopeTree::new();
        tree.register(1, None);
        tree.register(2, Some(1));
        tree.register(3, Some(2));

        assert!(tree.is_descendant_of(3, 1));
        assert!(!tree.is_descendant_of(1, 3));
    }
}
