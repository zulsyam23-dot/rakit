use std::path::{Path, PathBuf};
use crate::{RakitManifest, Result};

/// Package builder — compile package dari source.
pub struct PackageBuilder {
    output_dir: PathBuf,
}

impl PackageBuilder {
    pub fn new(output_dir: PathBuf) -> Self {
        PackageBuilder { output_dir }
    }

    pub fn build(&self, _source_dir: &Path, manifest: &RakitManifest) -> Result<PathBuf> {
        let target_dir = self.output_dir.join(&manifest.package.name);
        std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

        let output = target_dir.join(format!("{}.rakit", manifest.package.name));
        std::fs::write(&output, format!(
            "// Package {} v{} — built by rakit-pm\n",
            manifest.package.name, manifest.package.version
        )).map_err(|e| e.to_string())?;

        Ok(output)
    }
}
