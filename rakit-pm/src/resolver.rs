use std::collections::HashMap;
use crate::registry::PackageRegistry;
use crate::{RakitManifest, VersionConstraint, ResolvedGraph, PackageVersion, Result};

pub struct DependencyResolver {
    registry: PackageRegistry,
}

impl DependencyResolver {
    pub fn new(registry: PackageRegistry) -> Self {
        DependencyResolver { registry }
    }

    pub fn resolve(&mut self, manifest: &RakitManifest) -> Result<ResolvedGraph> {
        let mut resolved: HashMap<String, PackageVersion> = HashMap::new();

        for (name, constraint_str) in &manifest.dependencies {
            let constraint = VersionConstraint::new(constraint_str);
            let best = self.find_best_match(name, &constraint)?;
            resolved.insert(name.clone(), best);
        }

        Ok(ResolvedGraph { packages: resolved })
    }

    fn find_best_match(&self, name: &str, constraint: &VersionConstraint) -> Result<PackageVersion> {
        let versions = self.registry.list_versions(name)?;

        let compatible: Vec<_> = versions.iter()
            .filter(|v| constraint.satisfies(&v.version))
            .collect();

        if compatible.is_empty() {
            return Err(format!(
                "Tidak ada version '{}' yang memenuhi constraint '{}'",
                name, constraint.raw
            ));
        }

        Ok(compatible.into_iter()
            .max_by_key(|v| v.version.clone())
            .unwrap()
            .clone())
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        DependencyResolver::new(PackageRegistry::default())
    }
}
