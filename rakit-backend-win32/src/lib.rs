pub mod control;
pub mod event;
pub mod ffi;
pub mod painter;
pub mod resources;
pub mod window;

use control::{tag_to_win32_class, tag_to_win32_style};
use painter::Win32Painter;
use rakit_runtime::event::{dispatch_event, EventData, EventType};
use rakit_ui::backend::*;
use rakit_vdom::node::AttrValue;
use std::collections::HashMap;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

#[allow(dead_code)]
pub struct Win32Backend {
    windows: HashMap<u64, Win32WindowData>,
    hinstance: isize,
    element_map: HashMap<u64, isize>,
    window_for_hwnd: HashMap<isize, u64>,
    next_elem_id: u64,
    painter: Win32Painter,
}

struct Win32WindowData {
    window: window::Win32Window,
    handle: u64,
}

impl Win32Backend {
    pub fn new() -> Self {
        Win32Backend {
            windows: HashMap::new(),
            hinstance: 0,
            element_map: HashMap::new(),
            window_for_hwnd: HashMap::new(),
            next_elem_id: 1,
            painter: Win32Painter::new(),
        }
    }

    fn get_hwnd(&self, elem: &u64) -> Option<isize> {
        self.element_map.get(elem).copied()
    }
}

impl UiBackend for Win32Backend {
    type WindowHandle = u64;
    type ElementHandle = u64;
    type FontHandle = isize;

    fn init(&mut self, _config: &AppConfig) -> Result<()> {
        unsafe {
            self.hinstance = ffi::GetModuleHandleW(std::ptr::null());
            if self.hinstance == 0 {
                return Err("Failed to get module handle".into());
            }
        }
        Ok(())
    }

    fn create_window(&mut self, config: &WindowConfig) -> Result<u64> {
        let win = window::Win32Window::create(config, self.hinstance)?;
        let handle = win.id();
        self.window_for_hwnd.insert(win.hwnd, handle);
        self.windows.insert(
            handle,
            Win32WindowData {
                window: win,
                handle,
            },
        );
        Ok(handle)
    }

    fn root_element(&self, window: &u64) -> u64 {
        *window
    }

    fn run_event_loop(&mut self) -> Result<()> {
        let mut msg = unsafe { std::mem::zeroed() };
        unsafe {
            while GetMessageW(&mut msg, 0, 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }

    fn quit(&mut self) {
        unsafe {
            PostQuitMessage(0);
        }
    }

    fn create_element(&mut self, window: &u64, tag: &str) -> u64 {
        let id = self.next_elem_id;
        self.next_elem_id += 1;

        if let Some(win_data) = self.windows.get_mut(window) {
            let class = tag_to_win32_class(tag);
            let style = tag_to_win32_style(tag);
            let hwnd = win_data.window.create_child(class, style, 0, 0, 100, 50, id);
            if hwnd != 0 {
                self.element_map.insert(id, hwnd);
            }
        }

        id
    }

    fn create_text(&mut self, window: &u64, text: &str) -> u64 {
        let id = self.next_elem_id;
        self.next_elem_id += 1;

        if let Some(win_data) = self.windows.get_mut(window) {
            let hwnd = win_data.window.create_child("STATIC", WS_CHILD | WS_VISIBLE, 0, 0, 0, 0, id);
            if hwnd != 0 {
                win_data.window.set_text(hwnd, text);
                self.element_map.insert(id, hwnd);
            }
        }

        id
    }

    fn set_attribute(&mut self, elem: &u64, name: &str, value: &AttrValue) {
        if let Some(hwnd) = self.get_hwnd(elem) {
            unsafe {
                match name {
                    "text" | "value" | "teks" | "label" => {
                        if let AttrValue::String(text) = value {
                            let wide: Vec<u16> = format!("{}\0", text).encode_utf16().collect();
                            SetWindowTextW(hwnd, wide.as_ptr());
                        }
                    }
                    "enabled" | "aktif" => {
                        if let AttrValue::Bool(b) = value {
                            ffi::EnableWindow(hwnd, if *b { 1 } else { 0 });
                        }
                    }
                    "visible" | "terlihat" => {
                        if let AttrValue::Bool(b) = value {
                            ShowWindow(hwnd, if *b { SW_SHOW } else { SW_HIDE });
                        }
                    }
                    "width" | "lebar" | "height" | "tinggi" => {
                    }
                    _ => {}
                }
            }
        }
    }

    fn remove_attribute(&mut self, elem: &u64, _name: &str) {
        let _ = elem;
    }

    fn set_text(&mut self, elem: &u64, text: &str) {
        if let Some(hwnd) = self.get_hwnd(elem) {
            unsafe {
                let wide: Vec<u16> = format!("{}\0", text).encode_utf16().collect();
                SetWindowTextW(hwnd, wide.as_ptr());
            }
        }
    }

    fn append_child(&mut self, _parent: &u64, _child: &u64) {
    }

    fn insert_child(&mut self, _parent: &u64, _child: &u64, _index: usize) {
    }

    fn remove_child(&mut self, _parent: &u64, child: &u64) {
        if let Some(hwnd) = self.get_hwnd(child) {
            unsafe {
                DestroyWindow(hwnd);
            }
            self.element_map.remove(child);
        }
    }

    fn move_child(&mut self, _parent: &u64, _child: &u64, _to_index: usize) {
    }

    fn attach_event(&mut self, elem: &u64, _event_type: EventType, _handler_id: u64) {
        let _ = elem;
    }

    fn detach_event(&mut self, _elem: &u64, _handler_id: u64) {}

    fn dispatch_event(&self, handler_id: u64, data: EventData) {
        dispatch_event(handler_id, data);
    }

    fn apply_stylesheet(&mut self, _window: &u64, _css: &str) {
    }

    fn set_style(&mut self, _elem: &u64, _property: &str, _value: &str) {
    }

    fn set_bounds(&mut self, elem: &u64, x: f64, y: f64, w: f64, h: f64) {
        if let Some(hwnd) = self.get_hwnd(elem) {
            unsafe {
                SetWindowPos(
                    hwnd,
                    0,
                    x as i32,
                    y as i32,
                    w as i32,
                    h as i32,
                    SWP_NOZORDER,
                );
            }
        }
    }

    fn measure(&mut self, _elem: &u64) -> (f64, f64) {
        (0.0, 0.0)
    }
}
