pub mod consumer;
pub mod provider;
pub mod scope;

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ContextId(u64);

pub struct Context<T: Clone + 'static> {
    pub name: String,
    pub default: T,
    pub id: ContextId,
}

impl<T: Clone + 'static> Context<T> {
    pub fn baru(name: &str, default: T) -> Rc<Self> {
        static NEXT_CTX_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Rc::new(Context {
            name: name.to_string(),
            default,
            id: ContextId(NEXT_CTX_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)),
        })
    }
}

thread_local! {
    static CONTEXT_TREE: RefCell<ContextTree> = RefCell::new(ContextTree::new());
}

pub struct ContextTree {
    providers: HashMap<u64, Vec<(ContextId, Box<dyn Any>)>>,
    resolve_cache: RefCell<HashMap<(u64, ContextId), Option<Box<dyn Any>>>>,
}

impl ContextTree {
    pub fn new() -> Self {
        ContextTree {
            providers: HashMap::new(),
            resolve_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn set_provider<T: 'static>(&mut self, fiber_id: u64, context_id: ContextId, value: T) {
        self.providers
            .entry(fiber_id)
            .or_insert_with(Vec::new)
            .push((context_id, Box::new(value)));
        self.resolve_cache.borrow_mut().clear();
    }

    pub fn get_value<T: Clone + 'static>(
        &self,
        fiber_id: u64,
        context: &Context<T>,
    ) -> T {
        let cache_key = (fiber_id, context.id);
        if let Some(cached) = self.resolve_cache.borrow().get(&cache_key) {
            if let Some(val) = cached {
                return val
                    .downcast_ref::<T>()
                    .cloned()
                    .unwrap_or_else(|| context.default.clone());
            }
            return context.default.clone();
        }

        let mut current_id = Some(fiber_id);
        while let Some(fid) = current_id {
            if let Some(providers) = self.providers.get(&fid) {
                for (ctx_id, value) in providers {
                    if *ctx_id == context.id {
                        let val = value
                            .downcast_ref::<T>()
                            .cloned()
                            .unwrap_or_else(|| context.default.clone());
                        self.resolve_cache
                            .borrow_mut()
                            .insert(cache_key, Some(Box::new(val.clone())));
                        return val;
                    }
                }
            }
            current_id = get_parent_fiber(fid);
        }

        self.resolve_cache.borrow_mut().insert(cache_key, None);
        context.default.clone()
    }
}

pub fn get_parent_fiber(fiber_id: u64) -> Option<u64> {
    FIBER_TREE.with(|ft| {
        let tree = ft.borrow();
        tree.get(&fiber_id).copied().flatten()
    })
}

thread_local! {
    pub static FIBER_TREE: RefCell<HashMap<u64, Option<u64>>> = RefCell::new(HashMap::new());
}

pub fn register_fiber(fiber_id: u64, parent: Option<u64>) {
    FIBER_TREE.with(|ft| {
        ft.borrow_mut().insert(fiber_id, parent);
    });
}

pub fn unregister_fiber(fiber_id: u64) {
    FIBER_TREE.with(|ft| {
        ft.borrow_mut().remove(&fiber_id);
    });
}

pub fn with_context_tree<F, R>(f: F) -> R
where
    F: FnOnce(&mut ContextTree) -> R,
{
    CONTEXT_TREE.with(|tree| f(&mut tree.borrow_mut()))
}

pub fn read_context_tree<F, R>(f: F) -> R
where
    F: FnOnce(&ContextTree) -> R,
{
    CONTEXT_TREE.with(|tree| f(&tree.borrow()))
}
