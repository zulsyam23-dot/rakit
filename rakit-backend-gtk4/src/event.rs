use rakit_runtime::event::EventType;

pub struct GtkEventMapper;

impl GtkEventMapper {
    pub fn gtk_signal_to_event_type(signal_name: &str) -> Option<EventType> {
        match signal_name {
            "clicked" => Some(EventType::Click),
            "activate" | "activated" => Some(EventType::Click),
            "changed" | "text-changed" => Some(EventType::Change),
            "focus-in-event" => Some(EventType::Focus),
            "focus-out-event" => Some(EventType::Blur),
            "key-pressed" => Some(EventType::KeyDown),
            "key-released" => Some(EventType::KeyUp),
            "motion-event" => Some(EventType::MouseMove),
            "button-press-event" => Some(EventType::MouseDown),
            "button-release-event" => Some(EventType::MouseUp),
            "scroll-event" => Some(EventType::Scroll),
            _ => None,
        }
    }

    pub fn event_type_to_gtk_signal(event_type: &EventType) -> &'static str {
        match event_type {
            EventType::Click => "clicked",
            EventType::DoubleClick => "button-press-event",
            EventType::Change => "changed",
            EventType::Input => "changed",
            EventType::Submit => "activate",
            EventType::Focus => "focus-in-event",
            EventType::Blur => "focus-out-event",
            EventType::KeyDown => "key-pressed",
            EventType::KeyUp => "key-released",
            EventType::KeyPress => "key-pressed",
            EventType::MouseMove => "motion-event",
            EventType::MouseDown => "button-press-event",
            EventType::MouseUp => "button-release-event",
            EventType::Scroll => "scroll-event",
            _ => "clicked",
        }
    }
}
