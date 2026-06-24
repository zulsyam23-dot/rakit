use crate::scheduler::SCHEDULER;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DevMetrics {
    pub total_renders: u64,
    pub total_patches: u64,
    pub total_diff_time_ms: f64,
    pub hook_counts: HashMap<String, usize>,
    pub render_times: Vec<f64>,
    pub component_render_counts: HashMap<String, u64>,
    pub memory_usage: Option<u64>,
    pub fps: f64,
}

pub struct RakitDevTools {
    #[allow(dead_code)]
    connected: bool,
    pub metrics: DevMetrics,
}

impl RakitDevTools {
    pub fn new() -> Self {
        RakitDevTools {
            connected: false,
            metrics: DevMetrics::default(),
        }
    }

    pub fn record_render(&mut self, component: &str, time_ms: f64) {
        self.metrics.total_renders += 1;
        self.metrics.render_times.push(time_ms);
        *self
            .metrics
            .component_render_counts
            .entry(component.to_string())
            .or_insert(0) += 1;

        if self.metrics.render_times.len() > 100 {
            self.metrics.render_times.remove(0);
        }
    }

    pub fn record_patch(&mut self) {
        self.metrics.total_patches += 1;
    }

    pub fn estimated_fps(&self) -> f64 {
        if self.metrics.render_times.len() < 2 {
            return 60.0;
        }
        let avg_time: f64 =
            self.metrics.render_times.iter().sum::<f64>() / self.metrics.render_times.len() as f64;
        1000.0 / avg_time.max(1.0)
    }

    pub fn print_stats(&self) {
        let avg_render = self.metrics.render_times.iter().sum::<f64>()
            / self.metrics.render_times.len().max(1) as f64;
        println!("╔══════════════════════════╗");
        println!("║   Rakit DevTools Stats   ║");
        println!("╠══════════════════════════╣");
        println!("║ Total renders: {:>6}    ║", self.metrics.total_renders);
        println!("║ Total patches: {:>6}    ║", self.metrics.total_patches);
        println!("║ Avg render:    {:>6.1}ms ║", avg_render);
        println!("║ Est. FPS:      {:>6.1}   ║", self.estimated_fps());
        println!("║ Components:              ║");
        for (name, count) in &self.metrics.component_render_counts {
            println!("║   {}: {} renders   ║", name, count);
        }
        println!("╚══════════════════════════╝");
    }
}

pub struct HotReloader {
    watch_paths: Vec<String>,
    component_registry: HashMap<String, String>,
    devtools: RakitDevTools,
}

impl HotReloader {
    pub fn new() -> Self {
        HotReloader {
            watch_paths: Vec::new(),
            component_registry: HashMap::new(),
            devtools: RakitDevTools::new(),
        }
    }

    pub fn watch(&mut self, path: &str) {
        self.watch_paths.push(path.to_string());
    }

    pub fn on_file_changed(&mut self, file_path: &str, _source: &str) {
        eprintln!("[HotReload] File changed: {}", file_path);
        for (name, _) in &self.component_registry.clone() {
            if let Some(fiber_id) = find_fiber_by_component(name) {
                SCHEDULER.with(|s| {
                    s.borrow_mut().schedule_update(fiber_id);
                });
            }
        }
    }

    pub fn register_component(&mut self, name: &str, source: &str) {
        self.component_registry
            .insert(name.to_string(), source.to_string());
    }

    pub fn devtools(&self) -> &RakitDevTools {
        &self.devtools
    }

    pub fn devtools_mut(&mut self) -> &mut RakitDevTools {
        &mut self.devtools
    }
}

fn find_fiber_by_component(name: &str) -> Option<u64> {
    SCHEDULER.with(|s| {
        let scheduler = s.borrow();
        for (id, fiber) in &scheduler.root.fibers {
            if fiber.tag == name {
                return Some(*id);
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devtools_metrics() {
        let mut devtools = RakitDevTools::new();
        devtools.record_render("App", 5.0);
        devtools.record_render("App", 8.0);
        assert_eq!(devtools.metrics.total_renders, 2);
        assert!(devtools.estimated_fps() > 0.0);
    }

    #[test]
    fn test_devtools_fps_estimation() {
        let mut devtools = RakitDevTools::new();
        for _ in 0..10 {
            devtools.record_render("App", 16.0);
        }
        let fps = devtools.estimated_fps();
        assert!(fps > 55.0 && fps < 65.0);
    }

    #[test]
    fn test_devtools_record_patch() {
        let mut devtools = RakitDevTools::new();
        devtools.record_patch();
        devtools.record_patch();
        assert_eq!(devtools.metrics.total_patches, 2);
    }

    #[test]
    fn test_hot_reloader_new() {
        let reloader = HotReloader::new();
        assert!(reloader.watch_paths.is_empty());
    }

    #[test]
    fn test_hot_reloader_watch() {
        let mut reloader = HotReloader::new();
        reloader.watch("src/main.rakit");
        assert_eq!(reloader.watch_paths.len(), 1);
    }

    #[test]
    fn test_hot_reloader_register_component() {
        let mut reloader = HotReloader::new();
        reloader.register_component("App", "komponen App() { ... }");
        assert_eq!(reloader.component_registry.len(), 1);
    }
}
