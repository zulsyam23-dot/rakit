pub mod callback;
pub mod effect;
pub mod memo;
pub mod reducer;
pub mod ref_cell;
pub mod refs;
pub mod state;

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub enum RakitValue {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    Object(Rc<RefCell<Vec<(String, RakitValue)>>>),
    Array(Rc<RefCell<Vec<RakitValue>>>),
    Component(Box<dyn Any>),
    Callback(Rc<dyn Fn(Vec<RakitValue>) -> RakitValue>),
}

impl Clone for RakitValue {
    fn clone(&self) -> Self {
        match self {
            RakitValue::Null => RakitValue::Null,
            RakitValue::Bool(b) => RakitValue::Bool(*b),
            RakitValue::Number(n) => RakitValue::Number(*n),
            RakitValue::Text(s) => RakitValue::Text(s.clone()),
            RakitValue::Object(o) => RakitValue::Object(Rc::new(RefCell::new(o.borrow().clone()))),
            RakitValue::Array(a) => RakitValue::Array(Rc::new(RefCell::new(a.borrow().clone()))),
            RakitValue::Component(_) => panic!("Cannot clone Component RakitValue"),
            RakitValue::Callback(c) => RakitValue::Callback(Rc::clone(c)),
        }
    }
}

impl fmt::Debug for RakitValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RakitValue::Null => write!(f, "Null"),
            RakitValue::Bool(b) => write!(f, "Bool({})", b),
            RakitValue::Number(n) => write!(f, "Number({})", n),
            RakitValue::Text(s) => write!(f, "Text({:?})", s),
            RakitValue::Object(_) => write!(f, "Object(...)"),
            RakitValue::Array(_) => write!(f, "Array(...)"),
            RakitValue::Component(_) => write!(f, "Component(...)"),
            RakitValue::Callback(_) => write!(f, "Callback(...)"),
        }
    }
}

impl RakitValue {
    pub fn as_text(&self) -> Option<String> {
        match self {
            RakitValue::Text(s) => Some(s.clone()),
            RakitValue::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            RakitValue::Number(n) => Some(*n),
            RakitValue::Text(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RakitValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            RakitValue::Null => false,
            RakitValue::Bool(b) => *b,
            RakitValue::Number(n) => *n != 0.0 && !n.is_nan(),
            RakitValue::Text(s) => !s.is_empty(),
            RakitValue::Object(_) => true,
            RakitValue::Array(a) => !a.borrow().is_empty(),
            RakitValue::Component(_) => true,
            RakitValue::Callback(_) => true,
        }
    }

    pub fn i32(&self) -> Option<i32> {
        self.as_number().map(|n| n as i32)
    }
}

impl From<&str> for RakitValue {
    fn from(s: &str) -> Self {
        RakitValue::Text(s.to_string())
    }
}

impl From<String> for RakitValue {
    fn from(s: String) -> Self {
        RakitValue::Text(s)
    }
}

impl From<f64> for RakitValue {
    fn from(n: f64) -> Self {
        RakitValue::Number(n)
    }
}

impl From<bool> for RakitValue {
    fn from(b: bool) -> Self {
        RakitValue::Bool(b)
    }
}

impl From<i32> for RakitValue {
    fn from(n: i32) -> Self {
        RakitValue::Number(n as f64)
    }
}

impl From<u64> for RakitValue {
    fn from(n: u64) -> Self {
        RakitValue::Number(n as f64)
    }
}

pub type HookState = Box<dyn Any>;

pub trait Hook {
    fn run(&mut self, ctx: &mut HookContext) -> RakitValue;
}

pub struct HookContext {
    pub hooks: Vec<HookState>,
    pub index: usize,
    pub current_hook: usize,
    pub fiber_id: u64,
}

impl HookContext {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            index: 0,
            current_hook: 0,
            fiber_id: 0,
        }
    }

    pub fn next_hook<T: 'static>(&mut self, init: T) -> &mut T {
        if self.index >= self.hooks.len() {
            self.hooks.push(Box::new(init));
        }
        let state = &mut self.hooks[self.index];
        let downcasted = state.downcast_mut::<T>().expect("Hook type mismatch");
        self.index += 1;
        downcasted
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.current_hook = 0;
    }

    pub fn clear(&mut self) {
        self.hooks.clear();
        self.index = 0;
        self.current_hook = 0;
    }
}
