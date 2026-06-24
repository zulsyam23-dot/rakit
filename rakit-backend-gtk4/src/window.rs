use rakit_ui::backend::WindowConfig;

pub struct GtkWindow {
    pub id: u64,
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub resizable: bool,
    pub decorated: bool,
}

impl GtkWindow {
    pub fn new(config: &WindowConfig, id: u64) -> Self {
        GtkWindow {
            id,
            title: config.title.clone(),
            width: config.width as i32,
            height: config.height as i32,
            resizable: config.resizable,
            decorated: config.decorated,
        }
    }

    pub fn config(&self) -> WindowConfig {
        let mut cfg = WindowConfig::new(&self.title, self.width as u32, self.height as u32);
        cfg.resizable = self.resizable;
        cfg.decorated = self.decorated;
        cfg
    }
}
