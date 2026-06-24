use rakit_runtime::event::EventType;

pub struct DomEventMapper;

impl DomEventMapper {
    pub fn event_type_to_js(event_type: &EventType) -> String {
        match event_type {
            EventType::Click => "click".into(),
            EventType::DoubleClick => "dblclick".into(),
            EventType::Change => "change".into(),
            EventType::Input => "input".into(),
            EventType::Submit => "submit".into(),
            EventType::Focus => "focus".into(),
            EventType::Blur => "blur".into(),
            EventType::KeyDown => "keydown".into(),
            EventType::KeyUp => "keyup".into(),
            EventType::KeyPress => "keypress".into(),
            EventType::MouseMove => "mousemove".into(),
            EventType::MouseDown => "mousedown".into(),
            EventType::MouseUp => "mouseup".into(),
            EventType::MouseEnter => "mouseenter".into(),
            EventType::MouseLeave => "mouseleave".into(),
            EventType::Scroll => "scroll".into(),
            EventType::Load => "load".into(),
            EventType::TouchStart => "touchstart".into(),
            EventType::TouchEnd => "touchend".into(),
            EventType::TouchMove => "touchmove".into(),
            EventType::Custom(name) => name.clone(),
        }
    }

    pub fn js_to_event_type(js_name: &str) -> EventType {
        match js_name {
            "click" => EventType::Click,
            "dblclick" => EventType::DoubleClick,
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
            _ => EventType::Custom(js_name.to_string()),
        }
    }
}
