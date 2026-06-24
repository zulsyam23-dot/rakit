use super::{HookContext, RakitValue};
use crate::hook::reducer::deps_equal;
use std::any::Any;

pub struct CallbackHook {
    pub callback: Box<dyn Any>,
    pub deps: Vec<RakitValue>,
    pub last_deps: Option<Vec<RakitValue>>,
}

pub fn use_callback<F: Clone + 'static>(
    ctx: &mut HookContext,
    callback: F,
    deps: Vec<RakitValue>,
) -> F {
    let idx = ctx.current_hook;
    ctx.current_hook += 1;

    if idx < ctx.hooks.len() {
        let hook = &ctx.hooks[idx];
        if let Some(cb_hook) = hook.downcast_ref::<CallbackHook>() {
            if let Some(last_deps) = &cb_hook.last_deps {
                if deps_equal(&deps, last_deps) {
                    if let Some(cb) = cb_hook.callback.downcast_ref::<F>() {
                        return cb.clone();
                    }
                }
            }
        }
    }

    if idx >= ctx.hooks.len() {
        ctx.hooks.push(Box::new(CallbackHook {
            callback: Box::new(callback.clone()),
            deps: deps.clone(),
            last_deps: Some(deps),
        }));
    } else {
        ctx.hooks[idx] = Box::new(CallbackHook {
            callback: Box::new(callback.clone()),
            deps: deps.clone(),
            last_deps: Some(deps),
        });
    }

    callback
}
