use rakit_ui::backend::Color;
use std::collections::HashMap;
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::*;

pub struct Win32Painter {
    fonts: HashMap<String, isize>,
}

impl Win32Painter {
    pub fn new() -> Self {
        Win32Painter {
            fonts: HashMap::new(),
        }
    }

    pub fn create_font(
        &mut self,
        family: &str,
        size: i32,
        bold: bool,
        italic: bool,
    ) -> isize {
        let key = format!("{}-{}-{}-{}", family, size, bold, italic);
        if let Some(font) = self.fonts.get(&key) {
            return *font;
        }

        unsafe {
            let font = CreateFontW(
                -size,
                0,
                0,
                0,
                if bold { 700 } else { 400 },
                italic as u32,
                0,
                0,
                1,
                0,
                0,
                0,
                0,
                std::ptr::null(),
            );
            self.fonts.insert(key, font);
            font
        }
    }

    pub fn set_bk_color(&self, hdc: isize, color: Color) {
        unsafe {
            SetBkColor(hdc, color_to_rgb(color));
        }
    }

    pub fn set_text_color(&self, hdc: isize, color: Color) {
        unsafe {
            SetTextColor(hdc, color_to_rgb(color));
        }
    }

    pub fn _draw_text(&self, hdc: isize, text: &str, x: i32, y: i32, w: i32, h: i32) {
        let text_wide: Vec<u16> = text.encode_utf16().collect();
        unsafe {
            DrawTextW(
                hdc,
                text_wide.as_ptr(),
                text_wide.len() as i32,
                &mut RECT {
                    left: x,
                    top: y,
                    right: x + w,
                    bottom: y + h,
                },
                0,
            );
        }
    }

    pub fn fill_rect(&self, hdc: isize, x: i32, y: i32, w: i32, h: i32, color: Color) {
        unsafe {
            let brush = CreateSolidBrush(color_to_rgb(color));
            let rect = RECT {
                left: x,
                top: y,
                right: x + w,
                bottom: y + h,
            };
            FillRect(hdc, &rect, brush);
            DeleteObject(brush);
        }
    }
}

impl Drop for Win32Painter {
    fn drop(&mut self) {
        unsafe {
            for (_, font) in self.fonts.drain() {
                DeleteObject(font);
            }
        }
    }
}

fn color_to_rgb(color: Color) -> u32 {
    (color.r as u32) | ((color.g as u32) << 8) | ((color.b as u32) << 16)
}
