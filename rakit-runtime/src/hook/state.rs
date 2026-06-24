use super::HookContext;

pub struct StateHookData<T> {
    pub value: T,
    pub pending_update: Option<T>,
}

impl<T> StateHookData<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: initial,
            pending_update: None,
        }
    }
}

pub fn use_state<T: 'static + Clone>(ctx: &mut HookContext, initial: T) -> (T, Box<dyn Fn(T)>) {
    let data: &mut StateHookData<T> = ctx.next_hook(StateHookData::new(initial));
    let value = data.value.clone();
    let data_ptr: *mut StateHookData<T> = data as *mut _;
    let setter = Box::new(move |new_value: T| {
        unsafe {
            (*data_ptr).pending_update = Some(new_value);
        }
    });
    (value, setter)
}
