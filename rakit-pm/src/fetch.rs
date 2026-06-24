use std::path::PathBuf;
use crate::registry::PackageRegistry;
use crate::{ResolvedGraph, Result};

/// Package downloader — mengunduh packages yang sudah di-resolve.
pub struct PackageFetcher {
    registry: PackageRegistry,
}

impl PackageFetcher {
    pub fn new(registry: PackageRegistry) -> Self {
        PackageFetcher { registry }
    }

    pub fn fetch_all(&self, graph: &ResolvedGraph) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        for (name, version) in &graph.packages {
            let path = self.registry.download(name, &version.version.to_string())?;
            paths.push(path);
        }

        Ok(paths)
    }
}
