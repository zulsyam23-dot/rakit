use super::HookContext;

pub struct EffectData {
    pub setup: Option<Box<dyn Fn() -> Option<Box<dyn Fn()>>>>,
    pub cleanup: Option<Box<dyn Fn()>>,
    pub deps: Vec<u64>,
    pub prev_deps: Vec<u64>,
    pub has_run: bool,
}

pub fn use_effect<F>(ctx: &mut HookContext, deps: Vec<u64>, setup: F)
where
    F: Fn() -> Option<Box<dyn Fn()>> + 'static,
{
    let boxed_setup: Option<Box<dyn Fn() -> Option<Box<dyn Fn()>>>> = Some(Box::new(setup));

    let data: &mut EffectData = ctx.next_hook(EffectData {
        setup: None,
        cleanup: None,
        deps: Vec::new(),
        prev_deps: Vec::new(),
        has_run: false,
    });

    data.deps = deps;
    data.setup = boxed_setup;

    let deps_changed = !data.has_run || data.deps != data.prev_deps;

    if deps_changed {
        if let Some(cleanup) = data.cleanup.take() {
            cleanup();
        }
        if let Some(ref setup) = data.setup {
            data.cleanup = setup();
        }
        data.prev_deps = data.deps.clone();
        data.has_run = true;
    }
}

#[allow(dead_code)]
pub fn use_effect_once<F>(ctx: &mut HookContext, setup: F)
where
    F: Fn() -> Option<Box<dyn Fn()>> + 'static,
{
    use_effect(ctx, vec![0], setup);
}
