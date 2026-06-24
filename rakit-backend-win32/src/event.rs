use rakit_runtime::event::{EventData, EventType};

pub fn win32_msg_to_event_type(msg: u32) -> Option<EventType> {
    match msg {
        0x0201 => Some(EventType::MouseDown),  // WM_LBUTTONDOWN
        0x0202 => Some(EventType::MouseUp),    // WM_LBUTTONUP
        0x0203 => Some(EventType::DoubleClick), // WM_LBUTTONDBLCLK
        0x0200 => Some(EventType::MouseMove),  // WM_MOUSEMOVE
        0x020A => Some(EventType::Scroll),     // WM_MOUSEWHEEL
        0x0100 => Some(EventType::KeyDown),    // WM_KEYDOWN
        0x0101 => Some(EventType::KeyUp),      // WM_KEYUP
        0x0102 => Some(EventType::KeyPress),   // WM_CHAR
        0x0007 => Some(EventType::Focus),      // WM_SETFOCUS
        0x0008 => Some(EventType::Blur),        // WM_KILLFOCUS
        _ => None,
    }
}

pub fn extract_mouse_data(lparam: i64) -> (f64, f64) {
    let x = (lparam as u32 & 0xFFFF) as f64;
    let y = ((lparam as u32 >> 16) & 0xFFFF) as f64;
    (x, y)
}

pub fn extract_key_data(wparam: u64) -> Option<String> {
    let key = wparam as u32;
    match key {
        0x08 => Some("Backspace".into()),
        0x09 => Some("Tab".into()),
        0x0D => Some("Enter".into()),
        0x1B => Some("Escape".into()),
        0x20 => Some(" ".into()),
        0x21..=0x28 => Some(match key {
            0x21 => "PageUp",
            0x22 => "PageDown",
            0x23 => "End",
            0x24 => "Home",
            0x25 => "ArrowLeft",
            0x26 => "ArrowUp",
            0x27 => "ArrowRight",
            0x28 => "ArrowDown",
            _ => "",
        }.into()),
        0x30..=0x39 => Some(((key - 0x30 + b'0' as u32) as u8 as char).to_string()),
        0x41..=0x5A => Some(((key - 0x41 + b'A' as u32) as u8 as char).to_string()),
        _ => None,
    }
}

pub struct Win32EventHandler;

impl Win32EventHandler {
    pub fn dispatch_click(control_id: u16) {
        let data = EventData::new(EventType::Click);
        rakit_runtime::event::dispatch_event(control_id as u64, data);
    }

    pub fn dispatch_change(control_id: u16) {
        let data = EventData::new(EventType::Change);
        rakit_runtime::event::dispatch_event(control_id as u64, data);
    }

    pub fn dispatch_key(control_id: u16, key: &str) {
        let data = EventData::new(EventType::KeyDown).with_key(key.to_string());
        rakit_runtime::event::dispatch_event(control_id as u64, data);
    }
}
