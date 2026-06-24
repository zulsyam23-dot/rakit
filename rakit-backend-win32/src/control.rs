use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub fn tag_to_win32_class(tag: &str) -> &'static str {
    match tag {
        "div" | "container" | "header" | "footer" | "nav" | "main" | "section" => "STATIC",
        "button" | "tombol" => "BUTTON",
        "text" | "span" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "label" => "STATIC",
        "input" | "textbox" => "EDIT",
        "checkbox" => "BUTTON",
        "radio" | "radiobutton" => "BUTTON",
        "list" | "listbox" => "LISTBOX",
        "dropdown" | "select" | "combobox" => "COMBOBOX",
        "scroll" | "scrollbar" => "SCROLLBAR",
        "progress" | "progressbar" => "PROGRESS_CLASS",
        "slider" | "trackbar" => "TRACKBAR_CLASS",
        "image" | "img" => "STATIC",
        _ => "STATIC",
    }
}

pub fn tag_to_win32_style(tag: &str) -> u32 {
    match tag {
        "button" | "tombol" => WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
        "checkbox" => WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX as u32,
        "radio" => WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON as u32,
        "input" | "textbox" => WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL as u32,
        "h1" | "h2" | "h3" => WS_CHILD | WS_VISIBLE,
        "text" | "span" | "p" => WS_CHILD | WS_VISIBLE,
        "div" | "container" => WS_CHILD | WS_VISIBLE,
        "image" | "img" => WS_CHILD | WS_VISIBLE,
        "progressbar" => WS_CHILD | WS_VISIBLE,
        _ => WS_CHILD | WS_VISIBLE,
    }
}

pub fn tag_is_button(tag: &str) -> bool {
    matches!(tag, "button" | "tombol" | "checkbox" | "radio" | "radiobutton")
}

pub fn tag_is_edit(tag: &str) -> bool {
    matches!(tag, "input" | "textbox")
}
