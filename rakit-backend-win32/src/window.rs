use rakit_ui::backend::WindowConfig;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::ffi;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

static WINDOW_COUNT: AtomicU64 = AtomicU64::new(0);

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

pub struct Win32Window {
    pub hwnd: isize,
    pub hinstance: isize,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub root_hwnd: isize,
    pub controls: HashMap<u64, isize>,
    id: u64,
}

pub fn encode_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

extern "system" fn window_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let _hdc = BeginPaint(hwnd, &mut ps);
                EndPaint(hwnd, &ps);
                0
            }
            WM_COMMAND => {
                let control_id = (wparam & 0xFFFF) as u16;
                let notification = ((wparam >> 16) & 0xFFFF) as u16;
                if notification == BN_CLICKED as u16 {
                    let handler_id = control_id as u64;
                    let data = rakit_runtime::event::EventData::new(
                        rakit_runtime::event::EventType::Click,
                    );
                    rakit_runtime::event::dispatch_event(handler_id, data);
                }
                0
            }
            WM_CTLCOLORSTATIC => {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_CTLCOLORBTN => {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

impl Win32Window {
    pub fn create(config: &WindowConfig, hinstance: isize) -> Result<Self, Box<dyn std::error::Error>> {
        let class_name = encode_wide("RakitWindow\0");

        unsafe {
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: 0,
                hCursor: LoadCursorW(0, IDC_ARROW as *const u16),
                hbrBackground: CreateSolidBrush(rgb(
                    config.background_color.r,
                    config.background_color.g,
                    config.background_color.b,
                )),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };
            RegisterClassW(&wc);

            let title = encode_wide(&format!("{}\0", config.title));
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                config.width as i32,
                config.height as i32,
                0,
                0,
                hinstance,
                std::ptr::null(),
            );

            if hwnd == 0 {
                return Err("Failed to create window".into());
            }

            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            let id = WINDOW_COUNT.fetch_add(1, Ordering::SeqCst);

            Ok(Win32Window {
                hwnd,
                hinstance,
                title: config.title.clone(),
                width: config.width,
                height: config.height,
                root_hwnd: hwnd,
                controls: HashMap::new(),
                id,
            })
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn create_child(
        &mut self,
        class: &str,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        control_id: u64,
    ) -> isize {
        unsafe {
            let class_wide = encode_wide(&format!("{}\0", class));
            let hwnd = CreateWindowExW(
                0,
                class_wide.as_ptr(),
                std::ptr::null(),
                style,
                x,
                y,
                w,
                h,
                self.hwnd,
                control_id as isize,
                self.hinstance,
                std::ptr::null(),
            );
            if hwnd != 0 {
                self.controls.insert(control_id, hwnd);
            }
            hwnd
        }
    }

    pub fn set_text(&self, hwnd: isize, text: &str) {
        unsafe {
            let text_wide = encode_wide(&format!("{}\0", text));
            SetWindowTextW(hwnd, text_wide.as_ptr());
        }
    }

    #[allow(dead_code)]
    pub fn show(&self, hwnd: isize, visible: bool) {
        unsafe {
            ShowWindow(hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }

    #[allow(dead_code)]
    pub fn enable(&self, hwnd: isize, enabled: bool) {
        unsafe {
            ffi::EnableWindow(hwnd, if enabled { 1 } else { 0 });
        }
    }
}
