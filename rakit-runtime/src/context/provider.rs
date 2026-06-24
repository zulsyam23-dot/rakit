use super::Context;
use std::rc::Rc;

pub fn register_provider<T: Clone + 'static>(fiber_id: u64, context: &Rc<Context<T>>, value: T) {
    super::with_context_tree(|tree| {
        tree.set_provider(fiber_id, context.id, value);
    });
}
