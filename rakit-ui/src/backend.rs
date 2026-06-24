use rakit_runtime::event::{EventData, EventType};
use rakit_vdom::node::AttrValue;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait UiBackend {
    type WindowHandle: Clone + Send;
    type ElementHandle: Clone + Send + PartialEq;
    type FontHandle: Clone;

    fn init(&mut self, config: &AppConfig) -> Result<()>;

    fn create_window(&mut self, config: &WindowConfig) -> Result<Self::WindowHandle>;

    fn root_element(&self, window: &Self::WindowHandle) -> Self::ElementHandle;

    fn run_event_loop(&mut self) -> Result<()>;

    fn quit(&mut self);

    fn create_element(
        &mut self,
        window: &Self::WindowHandle,
        tag: &str,
    ) -> Self::ElementHandle;

    fn create_text(&mut self, window: &Self::WindowHandle, text: &str) -> Self::ElementHandle;

    fn set_attribute(&mut self, elem: &Self::ElementHandle, name: &str, value: &AttrValue);

    fn remove_attribute(&mut self, elem: &Self::ElementHandle, name: &str);

    fn set_text(&mut self, elem: &Self::ElementHandle, text: &str);

    fn append_child(
        &mut self,
        parent: &Self::ElementHandle,
        child: &Self::ElementHandle,
    );

    fn insert_child(
        &mut self,
        parent: &Self::ElementHandle,
        child: &Self::ElementHandle,
        index: usize,
    );

    fn remove_child(
        &mut self,
        parent: &Self::ElementHandle,
        child: &Self::ElementHandle,
    );

    fn move_child(
        &mut self,
        parent: &Self::ElementHandle,
        child: &Self::ElementHandle,
        to_index: usize,
    );

    fn attach_event(
        &mut self,
        elem: &Self::ElementHandle,
        event_type: EventType,
        handler_id: u64,
    );

    fn detach_event(&mut self, elem: &Self::ElementHandle, handler_id: u64);

    fn dispatch_event(&self, handler_id: u64, data: EventData);

    fn apply_stylesheet(&mut self, window: &Self::WindowHandle, css: &str);

    fn set_style(&mut self, elem: &Self::ElementHandle, property: &str, value: &str);

    fn set_bounds(
        &mut self,
        elem: &Self::ElementHandle,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    );

    fn measure(&mut self, elem: &Self::ElementHandle) -> (f64, f64);
}

pub struct AppConfig {
    pub app_name: String,
    pub org_name: String,
    pub version: String,
}

impl AppConfig {
    pub fn new(app_name: &str, org_name: &str, version: &str) -> Self {
        AppConfig {
            app_name: app_name.to_string(),
            org_name: org_name.to_string(),
            version: version.to_string(),
        }
    }
}

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub decorated: bool,
    pub background_color: Color,
}

impl WindowConfig {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        WindowConfig {
            title: title.to_string(),
            width,
            height,
            resizable: true,
            decorated: true,
            background_color: Color::default(),
        }
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub fn white() -> Self {
        Color::rgb(255, 255, 255)
    }

    pub fn black() -> Self {
        Color::rgb(0, 0, 0)
    }

    pub fn transparent() -> Self {
        Color::new(0, 0, 0, 0)
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;
        Color { r, g, b, a: 255 }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::white()
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    pub family: String,
    pub size: f64,
    pub weight: FontWeight,
    pub italic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Thin,
    Light,
    Normal,
    Medium,
    Bold,
    Black,
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Normal
    }
}

impl Font {
    pub fn new(family: &str, size: f64) -> Self {
        Font {
            family: family.to_string(),
            size,
            weight: FontWeight::Normal,
            italic: false,
        }
    }

    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
}
