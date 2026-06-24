use crate::component::Component;
use crate::hook::HookContext;
use std::collections::HashMap;

pub type FiberId = u64;

pub struct Fiber {
    pub id: FiberId,
    pub parent: Option<FiberId>,
    pub children: Vec<FiberId>,
    pub component: Option<Box<dyn Component>>,
    pub hook_ctx: HookContext,
    pub tag: String,
    pub props: HashMap<String, String>,
    pub dom_ref: Option<Box<dyn std::any::Any>>,
    pub dirty: bool,
    pub depth: u32,
}

impl Fiber {
    pub fn new(id: FiberId, tag: &str, depth: u32) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            component: None,
            hook_ctx: HookContext::new(),
            tag: tag.to_string(),
            props: HashMap::new(),
            dom_ref: None,
            dirty: true,
            depth,
        }
    }

    pub fn set_props(&mut self, props: Vec<(String, String)>) {
        for (k, v) in props {
            self.props.insert(k, v);
        }
    }

    pub fn render(&mut self) {
        if let Some(ref mut comp) = self.component {
            self.hook_ctx.reset();
            self.hook_ctx.fiber_id = self.id;
            comp.render(&mut self.hook_ctx);
        }
    }
}

pub struct FiberRoot {
    pub fibers: HashMap<FiberId, Fiber>,
    pub next_id: FiberId,
    pub root_id: FiberId,
}

impl FiberRoot {
    pub fn new() -> Self {
        Self {
            fibers: HashMap::new(),
            next_id: 1,
            root_id: 0,
        }
    }

    pub fn create_fiber(&mut self, tag: &str, parent: Option<FiberId>, depth: u32) -> FiberId {
        let id = self.next_id;
        self.next_id += 1;

        let mut fiber = Fiber::new(id, tag, depth);
        fiber.parent = parent;

        if let Some(pid) = parent {
            if let Some(parent_fiber) = self.fibers.get_mut(&pid) {
                parent_fiber.children.push(id);
            }
        }

        if parent.is_none() {
            self.root_id = id;
        }

        self.fibers.insert(id, fiber);
        id
    }

    pub fn remove_fiber(&mut self, id: FiberId) {
        self.fibers.remove(&id);
    }

    pub fn get_fiber(&self, id: FiberId) -> Option<&Fiber> {
        self.fibers.get(&id)
    }

    pub fn get_fiber_mut(&mut self, id: FiberId) -> Option<&mut Fiber> {
        self.fibers.get_mut(&id)
    }

    pub fn mark_dirty(&mut self, id: FiberId) {
        if let Some(fiber) = self.fibers.get_mut(&id) {
            fiber.dirty = true;
        }
    }

    pub fn root(&self) -> Option<&Fiber> {
        self.fibers.get(&self.root_id)
    }

    pub fn root_mut(&mut self) -> Option<&mut Fiber> {
        self.fibers.get_mut(&self.root_id)
    }
}
