use super::HookContext;

pub struct MemoData<T: Clone> {
    pub value: Option<T>,
    pub deps: Vec<u64>,
    pub prev_deps: Vec<u64>,
    pub has_run: bool,
}

pub fn use_memo<T: Clone + 'static>(
    ctx: &mut HookContext,
    deps: Vec<u64>,
    factory: impl Fn() -> T,
) -> Option<T> {
    let data: &mut MemoData<T> = ctx.next_hook(MemoData {
        value: None,
        deps: Vec::new(),
        prev_deps: Vec::new(),
        has_run: false,
    });

    data.deps = deps;

    let deps_changed = !data.has_run || data.deps != data.prev_deps;

    if deps_changed {
        data.value = Some(factory());
        data.prev_deps = data.deps.clone();
        data.has_run = true;
    }

    data.value.clone()
}

#[allow(dead_code)]
pub fn use_callback<T: Clone + 'static>(
    ctx: &mut HookContext,
    deps: Vec<u64>,
    factory: impl Fn() -> T,
) -> Option<T> {
    use_memo(ctx, deps, factory)
}
