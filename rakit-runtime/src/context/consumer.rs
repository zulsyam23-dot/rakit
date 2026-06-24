use super::Context;
use crate::hook::{HookContext, RakitValue};
use std::rc::Rc;

#[allow(non_snake_case)]
pub fn useContext<T: Clone + 'static>(
    ctx: &mut HookContext,
    context: &Rc<Context<T>>,
) -> T {
    let _idx = ctx.current_hook;
    ctx.current_hook += 1;
    let fiber_id = ctx.fiber_id;
    super::read_context_tree(|tree| tree.get_value(fiber_id, context))
}

#[allow(non_snake_case)]
pub fn useContextAsRakitValue<T: Clone + 'static>(
    ctx: &mut HookContext,
    context: &Rc<Context<T>>,
) -> RakitValue
where
    RakitValue: From<T>,
{
    let value: T = useContext(ctx, context);
    RakitValue::from(value)
}
