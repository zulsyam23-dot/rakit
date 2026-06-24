use std::path::{Path, PathBuf};
use crate::{PackageInfo, Result};

pub struct PackageRegistry {
    pub url: String,
    pub cache_dir: PathBuf,
}

impl PackageRegistry {
    pub fn default() -> Self {
        PackageRegistry {
            url: "https://registry.rakit.dev".to_string(),
            cache_dir: Path::new(".rakit-cache").join("packages"),
        }
    }

    pub fn new(url: &str, cache_dir: PathBuf) -> Self {
        PackageRegistry {
            url: url.to_string(),
            cache_dir,
        }
    }

    pub fn search(&self, query: &str) -> Result<Vec<PackageInfo>> {
        let _url = format!("{}/api/search?q={}", self.url, query);
        Ok(vec![
            PackageInfo {
                name: "rakit/ui".into(),
                version: "1.0.0".into(),
                description: "Komponen UI dasar Rakit".into(),
                authors: vec!["Tim Rakit".into()],
                license: "MIT".into(),
                dependencies: std::collections::HashMap::new(),
            },
            PackageInfo {
                name: "rakit/http".into(),
                version: "0.2.0".into(),
                description: "HTTP client untuk Rakit".into(),
                authors: vec!["Tim Rakit".into()],
                license: "MIT".into(),
                dependencies: std::collections::HashMap::new(),
            },
        ])
    }

    pub fn list_versions(&self, name: &str) -> Result<Vec<crate::PackageVersion>> {
        Ok(vec![
            crate::PackageVersion {
                name: name.to_string(),
                version: "1.0.0".to_string(),
            },
            crate::PackageVersion {
                name: name.to_string(),
                version: "0.9.0".to_string(),
            },
        ])
    }

    pub fn download(&self, name: &str, version: &str) -> Result<PathBuf> {
        let cache_path = self.cache_dir.join(name).join(version);

        if cache_path.exists() {
            return Ok(cache_path);
        }

        std::fs::create_dir_all(&cache_path).map_err(|e| e.to_string())?;

        let src_path = cache_path.join("src");
        std::fs::create_dir_all(&src_path).map_err(|e| e.to_string())?;

        let main_rakit = src_path.join("mod.rakit");
        std::fs::write(&main_rakit, format!(
            "// Package: {}\n// Version: {}\n\nfungsi init() -> I32 {{\n    0\n}}\n",
            name, version
        )).map_err(|e| e.to_string())?;

        Ok(cache_path)
    }
}
