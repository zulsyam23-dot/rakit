pub mod registry;
pub mod resolver;
pub mod fetch;
pub mod build;

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RakitManifest {
    pub package: PackageMeta,
    pub dependencies: HashMap<String, String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
}

#[derive(Debug, Clone)]
pub struct PackageVersion {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct VersionConstraint {
    pub raw: String,
}

impl VersionConstraint {
    pub fn new(raw: &str) -> Self {
        VersionConstraint { raw: raw.to_string() }
    }

    pub fn satisfies(&self, version: &str) -> bool {
        if self.raw == "*" || self.raw == "x" {
            return true;
        }
        if self.raw.starts_with('^') {
            let min = &self.raw[1..];
            return version.starts_with(min);
        }
        if self.raw.starts_with('~') {
            let req = &self.raw[1..];
            return version.starts_with(req);
        }
        self.raw == version
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    pub packages: HashMap<String, PackageVersion>,
}

pub type Result<T> = std::result::Result<T, String>;
