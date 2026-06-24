use super::HookContext;
use std::cell::RefCell;
use std::rc::Rc;

pub struct RefValue<T: Clone + 'static> {
    pub current: Rc<RefCell<T>>,
}

impl<T: Clone + 'static> RefValue<T> {
    pub fn get(&self) -> T {
        self.current.borrow().clone()
    }

    pub fn set(&self, value: T) {
        *self.current.borrow_mut() = value;
    }
}

impl<T: Clone + 'static> Clone for RefValue<T> {
    fn clone(&self) -> Self {
        RefValue {
            current: Rc::clone(&self.current),
        }
    }
}

struct RefHook<T: Clone + 'static> {
    cell: Rc<RefCell<T>>,
}

pub fn use_ref<T: Clone + 'static>(ctx: &mut HookContext, initial: T) -> RefValue<T> {
    let idx = ctx.current_hook;
    ctx.current_hook += 1;

    if idx < ctx.hooks.len() {
        let hook = &ctx.hooks[idx];
        if let Some(ref_hook) = hook.downcast_ref::<RefHook<T>>() {
            return RefValue {
                current: Rc::clone(&ref_hook.cell),
            };
        }
        panic!("Hook mismatch: expected Ref at {}", idx);
    }

    let cell = Rc::new(RefCell::new(initial));

    if idx >= ctx.hooks.len() {
        ctx.hooks.push(Box::new(RefHook { cell: Rc::clone(&cell) }));
    }

    RefValue { current: cell }
}
