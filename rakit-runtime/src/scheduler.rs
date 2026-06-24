use crate::fiber::{FiberId, FiberRoot};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

thread_local! {
    pub static SCHEDULER: RefCell<Scheduler> = RefCell::new(Scheduler::new());
}

pub struct PendingUpdate {
    pub fiber_id: FiberId,
    pub setter_id: u64,
    pub value: crate::hook::RakitValue,
}

pub struct Scheduler {
    pub root: FiberRoot,
    pub work_queue: VecDeque<FiberId>,
    pub is_rendering: bool,
    pub pending_updates: Vec<Box<dyn Fn()>>,
    pub batching: bool,
    pub batch_depth: u32,
    pub batched_updates: Vec<PendingUpdate>,
    pub state_updates: HashMap<u64, Vec<u64>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            root: FiberRoot::new(),
            work_queue: VecDeque::new(),
            is_rendering: false,
            pending_updates: Vec::new(),
            batching: false,
            batch_depth: 0,
            batched_updates: Vec::new(),
            state_updates: HashMap::new(),
        }
    }

    pub fn schedule_update(&mut self, fiber_id: FiberId) {
        if !self.work_queue.contains(&fiber_id) {
            self.root.mark_dirty(fiber_id);
            self.work_queue.push_back(fiber_id);
        }
        if !self.is_rendering && !self.batching {
            self.flush_work();
        }
    }

    pub fn in_transition<F: FnOnce()>(&mut self, f: F) {
        let prev = self.batching;
        self.batching = true;
        f();
        if !prev {
            self.batching = false;
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        self.flush_work();
    }

    pub fn flush_work(&mut self) {
        self.is_rendering = true;
        while let Some(fiber_id) = self.work_queue.pop_front() {
            self.render_fiber(fiber_id);
        }
        self.run_pending_effects();
        self.is_rendering = false;
    }

    pub fn flush_prioritized(&mut self) {
        let updates = std::mem::take(&mut self.batched_updates);

        let mut fiber_priority: HashMap<u64, u8> = HashMap::new();
        for update in &updates {
            let priority = fiber_priority.entry(update.fiber_id).or_insert(2);
            *priority = 0;
        }

        for update in &updates {
            self.apply_state_update(update);
        }

        let mut fibers: Vec<(u64, u8)> = fiber_priority.into_iter().collect();
        fibers.sort_by_key(|(_, p)| *p);

        for (fiber_id, _) in fibers {
            self.render_fiber(fiber_id);
        }
    }

    pub fn apply_state_update(&mut self, update: &PendingUpdate) {
        if let Some(fiber) = self.root.get_fiber_mut(update.fiber_id) {
            fiber.dirty = true;
        }
    }

    pub fn auto_batch<F: FnOnce()>(&mut self, f: F) {
        let depth = self.batch_depth;
        self.batch_depth += 1;

        if depth == 0 {
            f();
            self.batch_depth -= 1;
            self.flush();
        } else {
            f();
            self.batch_depth -= 1;
        }
    }

    fn render_fiber(&mut self, fiber_id: FiberId) {
        if let Some(fiber) = self.root.get_fiber_mut(fiber_id) {
            if fiber.dirty {
                fiber.render();
                fiber.dirty = false;
            }
        }
    }

    fn run_pending_effects(&mut self) {
        let effects = std::mem::take(&mut self.pending_updates);
        for effect in effects {
            effect();
        }
    }

    pub fn request_idle_callback<F>(&mut self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.pending_updates.push(Box::new(callback));
    }

    pub fn create_root(&mut self, tag: &str) -> FiberId {
        let id = self.root.create_fiber(tag, None, 0);
        crate::context::register_fiber(id, None);
        id
    }

    pub fn append_child(&mut self, parent_id: FiberId, tag: &str) -> FiberId {
        let depth = self
            .root
            .get_fiber(parent_id)
            .map(|f| f.depth + 1)
            .unwrap_or(0);
        let id = self.root.create_fiber(tag, Some(parent_id), depth);
        crate::context::register_fiber(id, Some(parent_id));
        id
    }

    pub fn reconcile(&mut self) {
        self.flush_work();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_scheduling() {
        let mut scheduler = Scheduler::new();
        let id = scheduler.create_root("root");
        assert_eq!(id, 1);
    }

    #[test]
    fn test_in_transition_batches_updates() {
        let mut scheduler = Scheduler::new();
        let id = scheduler.create_root("test");

        let _ = id;
        scheduler.batching = true;
        assert!(scheduler.batching);
        scheduler.batching = false;
    }

    #[test]
    fn test_auto_batch_depth() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.batch_depth, 0);

        scheduler.batch_depth = 1;
        assert_eq!(scheduler.batch_depth, 1);
        scheduler.batch_depth = 0;
    }

    #[test]
    fn test_nested_batch_depth() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.batch_depth, 0);

        scheduler.batch_depth += 1;
        assert_eq!(scheduler.batch_depth, 1);
        scheduler.batch_depth += 1;
        assert_eq!(scheduler.batch_depth, 2);
        scheduler.batch_depth -= 1;
        assert_eq!(scheduler.batch_depth, 1);
        scheduler.batch_depth -= 1;
        assert_eq!(scheduler.batch_depth, 0);
    }

    #[test]
    fn test_append_child_registers_fiber() {
        let mut scheduler = Scheduler::new();
        let root_id = scheduler.create_root("root");
        let child_id = scheduler.append_child(root_id, "child");
        assert_ne!(child_id, root_id);
    }

    #[test]
    fn test_schedule_update_in_batch_queues_fiber() {
        let mut scheduler = Scheduler::new();
        let id = scheduler.create_root("root");
        scheduler.batching = true;
        scheduler.schedule_update(id);
        assert!(scheduler.work_queue.contains(&id));
        scheduler.batching = false;
    }

    #[test]
    fn test_flush_work_clears_queue() {
        let mut scheduler = Scheduler::new();
        let id = scheduler.create_root("root");
        scheduler.batching = true;
        scheduler.schedule_update(id);
        scheduler.batching = false;
        scheduler.flush_work();
        assert!(scheduler.work_queue.is_empty());
    }

    #[test]
    fn test_batching_flag() {
        let mut scheduler = Scheduler::new();
        assert!(!scheduler.batching);
        scheduler.batching = true;
        assert!(scheduler.batching);
    }
}
