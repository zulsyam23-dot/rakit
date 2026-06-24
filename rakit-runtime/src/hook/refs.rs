use super::HookContext;
use std::cell::RefCell;
use std::rc::Rc;

pub struct RefData<T: Clone> {
    pub current: Rc<RefCell<T>>,
}

pub fn use_ref<T: Clone + 'static>(ctx: &mut HookContext, initial: T) -> Rc<RefCell<T>> {
    let data: &mut RefData<T> = ctx.next_hook(RefData {
        current: Rc::new(RefCell::new(initial)),
    });
    data.current.clone()
}
