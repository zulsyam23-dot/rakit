use std::collections::HashMap;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub struct ResourceManager {
    icons: HashMap<String, isize>,
    cursors: HashMap<String, isize>,
    bitmaps: HashMap<String, isize>,
    brushes: HashMap<u32, isize>,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            icons: HashMap::new(),
            cursors: HashMap::new(),
            bitmaps: HashMap::new(),
            brushes: HashMap::new(),
        }
    }

    pub fn _load_icon(&mut self, name: &str, hinstance: isize) -> isize {
        let name_wide: Vec<u16> = format!("{}\0", name).encode_utf16().collect();
        unsafe {
            let icon = LoadIconW(hinstance, name_wide.as_ptr());
            if icon != 0 {
                self.icons.insert(name.to_string(), icon);
            }
            icon
        }
    }

    pub fn _load_cursor(&mut self, name: &str) -> isize {
        unsafe {
            let id = match name {
                "hand" => IDC_HAND,
                "cross" => IDC_CROSS,
                "ibeam" | "text" => IDC_IBEAM,
                "wait" => IDC_WAIT,
                "resize_ns" => IDC_SIZENS,
                "resize_ew" => IDC_SIZEWE,
                _ => IDC_ARROW,
            };
            let cursor = LoadCursorW(0, id as *const u16);
            if cursor != 0 {
                self.cursors.insert(name.to_string(), cursor);
            }
            cursor
        }
    }

    pub fn _create_solid_brush(&mut self, color: u32) -> isize {
        if let Some(brush) = self.brushes.get(&color) {
            return *brush;
        }
        unsafe {
            let brush = CreateSolidBrush(color);
            self.brushes.insert(color, brush);
            brush
        }
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        unsafe {
            for (_, icon) in self.icons.drain() {
                DestroyIcon(icon);
            }
            for (_, _bitmap) in self.bitmaps.drain() {
            }
            for (_, brush) in self.brushes.drain() {
                DeleteObject(brush);
            }
        }
    }
}
