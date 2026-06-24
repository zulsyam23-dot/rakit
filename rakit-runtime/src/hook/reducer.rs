use super::{HookContext, RakitValue};
use crate::scheduler::SCHEDULER;
use std::any::Any;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static REDUCER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_id() -> u64 {
    REDUCER_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct ReducerHook {
    pub state: Box<dyn Any>,
    pub reducer_id: u64,
}

pub fn use_reducer<S: Clone + 'static, A: Clone + 'static>(
    ctx: &mut HookContext,
    reducer: Box<dyn Fn(S, A) -> S>,
    initial: S,
    init: Option<Box<dyn Fn() -> S>>,
) -> (S, Rc<dyn Fn(A)>) {
    let idx = ctx.current_hook;
    ctx.current_hook += 1;

    if idx < ctx.hooks.len() {
        let hook = &ctx.hooks[idx];
        if let Some(reducer_hook) = hook.downcast_ref::<ReducerHook>() {
            if let Some(state) = reducer_hook.state.downcast_ref::<S>() {
                let dispatch = create_dispatch(ctx.fiber_id, idx, reducer);
                return (state.clone(), dispatch);
            }
        }
        panic!("Hook mismatch: expected Reducer at {}", idx);
    }

    let state = match init {
        Some(f) => f(),
        None => initial,
    };

    let dispatch = create_dispatch(ctx.fiber_id, idx, reducer);

    if idx >= ctx.hooks.len() {
        ctx.hooks.push(Box::new(ReducerHook {
            state: Box::new(state.clone()),
            reducer_id: generate_id(),
        }));
    }

    (state, dispatch)
}

fn create_dispatch<S: Clone + 'static, A: Clone + 'static>(
    fiber_id: u64,
    hook_idx: usize,
    reducer: Box<dyn Fn(S, A) -> S>,
) -> Rc<dyn Fn(A)> {
    Rc::new(move |action: A| {
        SCHEDULER.with(|s| {
            let mut scheduler = s.borrow_mut();
            let fiber = scheduler.root.get_fiber(fiber_id);
            if let Some(f) = fiber {
                let hook_ctx = &f.hook_ctx;
                if hook_idx < hook_ctx.hooks.len() {
                    if let Some(reducer_hook) = hook_ctx.hooks[hook_idx].downcast_ref::<ReducerHook>()
                    {
                        if let Some(old_state) = reducer_hook.state.downcast_ref::<S>() {
                            let new_state = reducer(old_state.clone(), action);

                            if let Some(f_mut) = scheduler.root.get_fiber_mut(fiber_id) {
                                if hook_idx < f_mut.hook_ctx.hooks.len() {
                                    if let Some(rh) = f_mut.hook_ctx.hooks[hook_idx]
                                        .downcast_mut::<ReducerHook>()
                                    {
                                        rh.state = Box::new(new_state.clone());
                                    }
                                }
                            }

                            scheduler.schedule_update(fiber_id);
                        }
                    }
                }
            }
        });
    })
}

pub fn deps_equal(a: &[RakitValue], b: &[RakitValue]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if !rakti_value_eq(x, y) {
            return false;
        }
    }
    true
}

fn rakti_value_eq(a: &RakitValue, b: &RakitValue) -> bool {
    match (a, b) {
        (RakitValue::Null, RakitValue::Null) => true,
        (RakitValue::Bool(x), RakitValue::Bool(y)) => x == y,
        (RakitValue::Number(x), RakitValue::Number(y)) => (x - y).abs() < f64::EPSILON,
        (RakitValue::Text(x), RakitValue::Text(y)) => x == y,
        (RakitValue::Object(x), RakitValue::Object(y)) => {
            let xb = x.borrow();
            let yb = y.borrow();
            xb.len() == yb.len()
                && xb.iter().zip(yb.iter()).all(|((xk, xv), (yk, yv))| {
                    xk == yk && rakti_value_eq(xv, yv)
                })
        }
        (RakitValue::Array(x), RakitValue::Array(y)) => {
            let xb = x.borrow();
            let yb = y.borrow();
            xb.len() == yb.len()
                && xb.iter().zip(yb.iter()).all(|(xv, yv)| rakti_value_eq(xv, yv))
        }
        _ => false,
    }
}
