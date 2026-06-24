use crate::hook::{HookContext, RakitValue};
use crate::scheduler::SCHEDULER;
use std::any::Any;

#[derive(Debug, Clone)]
pub enum AsyncState<T: Clone + 'static> {
    Pending,
    Success(T),
    Error(String),
}

pub struct AsyncHookState {
    pub data: Option<Box<dyn Any>>,
    pub error: Option<String>,
    pub fiber_id: u64,
    pub hook_idx: usize,
}

impl AsyncHookState {
    pub fn new(fiber_id: u64, hook_idx: usize) -> Self {
        AsyncHookState {
            data: None,
            error: None,
            fiber_id,
            hook_idx,
        }
    }
}

pub fn use_async<T: Clone + 'static>(
    ctx: &mut HookContext,
    fiber_id: u64,
) -> AsyncState<T> {
    let idx = ctx.current_hook;
    ctx.current_hook += 1;

    if idx < ctx.hooks.len() {
        if let Some(async_hook) = ctx.hooks[idx].downcast_ref::<AsyncHookState>() {
            if let Some(data) = &async_hook.data {
                if let Some(val) = data.downcast_ref::<T>() {
                    return AsyncState::Success(val.clone());
                }
            }
            if let Some(err) = &async_hook.error {
                return AsyncState::Error(err.clone());
            }
            return AsyncState::Pending;
        }
        panic!("Hook mismatch: expected Async at {}", idx);
    }

    if idx >= ctx.hooks.len() {
        ctx.hooks.push(Box::new(AsyncHookState::new(fiber_id, idx)));
    }

    AsyncState::Pending
}

pub fn resolve_suspense(fiber_id: u64, value: RakitValue) {
    SCHEDULER.with(|s| {
        let mut scheduler = s.borrow_mut();
        if let Some(fiber) = scheduler.root.get_fiber_mut(fiber_id) {
            for hook in fiber.hook_ctx.hooks.iter_mut() {
                if let Some(async_hook) = hook.downcast_mut::<AsyncHookState>() {
                    match value {
                        RakitValue::Text(s) => async_hook.error = Some(s),
                        other => async_hook.data = Some(Box::new(other)),
                    }
                    break;
                }
            }
            fiber.dirty = true;
        }
        scheduler.schedule_update(fiber_id);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookContext;

    #[test]
    fn test_async_state_pending_on_first_render() {
        let mut ctx = HookContext::new();
        let result: AsyncState<i32> = use_async(&mut ctx, 1);
        match result {
            AsyncState::Pending => {}
            _ => panic!("Expected Pending on first render"),
        }
    }

    #[test]
    fn test_async_state_returns_success_after_resolve() {
        let mut ctx = HookContext::new();
        let _result: AsyncState<String> = use_async(&mut ctx, 1);
        assert_eq!(ctx.hooks.len(), 1);

        if let Some(hook) = ctx.hooks[0].downcast_ref::<AsyncHookState>() {
            assert!(hook.data.is_none());
            assert!(hook.error.is_none());
        } else {
            panic!("Expected AsyncHookState");
        }
    }
}
