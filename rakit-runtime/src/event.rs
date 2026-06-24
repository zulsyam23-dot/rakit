use crate::fiber::FiberId;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static GLOBAL_HANDLER_MAP: RefCell<HashMap<u64, Box<dyn Fn(EventData)>>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    Click,
    DoubleClick,
    Change,
    Input,
    Submit,
    Focus,
    Blur,
    KeyDown,
    KeyUp,
    KeyPress,
    MouseMove,
    MouseDown,
    MouseUp,
    MouseEnter,
    MouseLeave,
    Scroll,
    Load,
    TouchStart,
    TouchEnd,
    TouchMove,
    Custom(String),
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::Click => "click",
            EventType::DoubleClick => "dblclick",
            EventType::Change => "change",
            EventType::Input => "input",
            EventType::Submit => "submit",
            EventType::Focus => "focus",
            EventType::Blur => "blur",
            EventType::KeyDown => "keydown",
            EventType::KeyUp => "keyup",
            EventType::KeyPress => "keypress",
            EventType::MouseMove => "mousemove",
            EventType::MouseDown => "mousedown",
            EventType::MouseUp => "mouseup",
            EventType::MouseEnter => "mouseenter",
            EventType::MouseLeave => "mouseleave",
            EventType::Scroll => "scroll",
            EventType::Load => "load",
            EventType::TouchStart => "touchstart",
            EventType::TouchEnd => "touchend",
            EventType::TouchMove => "touchmove",
            EventType::Custom(name) => name,
        }
    }
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "click" => EventType::Click,
            "dblclick" | "doubleclick" => EventType::DoubleClick,
            "change" => EventType::Change,
            "input" => EventType::Input,
            "submit" => EventType::Submit,
            "focus" => EventType::Focus,
            "blur" => EventType::Blur,
            "keydown" => EventType::KeyDown,
            "keyup" => EventType::KeyUp,
            "keypress" => EventType::KeyPress,
            "mousemove" => EventType::MouseMove,
            "mousedown" => EventType::MouseDown,
            "mouseup" => EventType::MouseUp,
            "mouseenter" => EventType::MouseEnter,
            "mouseleave" => EventType::MouseLeave,
            "scroll" => EventType::Scroll,
            "load" => EventType::Load,
            "touchstart" => EventType::TouchStart,
            "touchend" => EventType::TouchEnd,
            "touchmove" => EventType::TouchMove,
            _ => EventType::Custom(s.to_string()),
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[allow(dead_code)]
pub type NativeEvent = String;

pub struct EventHandler {
    pub fiber_id: FiberId,
    pub event_type: EventType,
    pub handler: Box<dyn Fn(EventData)>,
}

#[derive(Debug, Clone)]
pub struct EventData {
    pub event_type: EventType,
    pub target: Option<FiberId>,
    pub target_id: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub mouse_x: Option<f64>,
    pub mouse_y: Option<f64>,
    pub prevent_default: bool,
    pub stop_propagation: bool,
}

impl EventData {
    pub fn new(event_type: EventType) -> Self {
        EventData {
            event_type,
            target: None,
            target_id: None,
            key: None,
            value: None,
            mouse_x: None,
            mouse_y: None,
            prevent_default: false,
            stop_propagation: false,
        }
    }

    pub fn with_target(mut self, target: FiberId) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    pub fn with_mouse(mut self, x: f64, y: f64) -> Self {
        self.mouse_x = Some(x);
        self.mouse_y = Some(y);
        self
    }

    pub fn prevent_default(mut self) -> Self {
        self.prevent_default = true;
        self
    }

    pub fn stop_propagation(mut self) -> Self {
        self.stop_propagation = true;
        self
    }
}

pub struct EventSystem {
    pub handlers: Vec<EventHandler>,
    pub capture_handlers: Vec<EventHandler>,
    pub event_map: HashMap<String, Vec<usize>>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            capture_handlers: Vec::new(),
            event_map: HashMap::new(),
        }
    }

    pub fn add_event_listener(
        &mut self,
        fiber_id: FiberId,
        event_type: &EventType,
        handler: Box<dyn Fn(EventData)>,
        use_capture: bool,
    ) -> usize {
        let handler = EventHandler {
            fiber_id,
            event_type: event_type.clone(),
            handler,
        };

        let idx = if use_capture {
            self.capture_handlers.push(handler);
            self.capture_handlers.len() - 1
        } else {
            self.handlers.push(handler);
            self.handlers.len() - 1
        };

        self.event_map
            .entry(event_type.to_string())
            .or_default()
            .push(idx);

        idx
    }

    pub fn dispatch(&self, event_data: EventData) {
        let event_type_str = event_data.event_type.to_string();
        let data_clone = event_data.clone();

        if let Some(indices) = self.event_map.get(&event_type_str) {
            for &idx in indices {
                if idx < self.capture_handlers.len() {
                    let handler = &self.capture_handlers[idx];
                    (handler.handler)(EventData {
                        event_type: event_data.event_type.clone(),
                        ..data_clone.clone()
                    });
                }
            }
        }

        if let Some(indices) = self.event_map.get(&event_type_str) {
            for &idx in indices {
                if idx < self.handlers.len() {
                    let handler = &self.handlers[idx];
                    (handler.handler)(EventData {
                        event_type: event_data.event_type.clone(),
                        ..data_clone.clone()
                    });
                }
            }
        }
    }

    pub fn remove_event_listener(&mut self, fiber_id: FiberId) {
        self.handlers.retain(|h| h.fiber_id != fiber_id);
        self.capture_handlers.retain(|h| h.fiber_id != fiber_id);
    }

    pub fn clear(&mut self) {
        self.handlers.clear();
        self.capture_handlers.clear();
        self.event_map.clear();
    }
}

/// Global handler registry — dipanggil oleh platform backends
pub fn register_global_handler(handler_id: u64, handler: Box<dyn Fn(EventData)>) {
    GLOBAL_HANDLER_MAP.with(|map| {
        map.borrow_mut().insert(handler_id, handler);
    });
}

pub fn unregister_global_handler(handler_id: u64) {
    GLOBAL_HANDLER_MAP.with(|map| {
        map.borrow_mut().remove(&handler_id);
    });
}

pub fn dispatch_event(handler_id: u64, data: EventData) {
    GLOBAL_HANDLER_MAP.with(|map| {
        let map = map.borrow();
        if let Some(handler) = map.get(&handler_id) {
            handler(data);
        }
    });
}
