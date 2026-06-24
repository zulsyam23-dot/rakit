use crate::hook::HookContext;

pub trait Component {
    fn render(&mut self, ctx: &mut HookContext);
    fn name(&self) -> &str;
}

pub type ComponentFactory = Box<dyn Fn() -> Box<dyn Component>>;
