use std::fs;
use std::path::{Path, PathBuf};

pub struct SnapshotTester {
    snapshot_dir: PathBuf,
    update_snapshots: bool,
}

impl SnapshotTester {
    pub fn new(dir: &str) -> Self {
        let update = std::env::args().any(|a| a == "--update" || a == "-u");
        SnapshotTester {
            snapshot_dir: Path::new("tests/snapshots").join(dir),
            update_snapshots: update,
        }
    }

    pub fn with_update(dir: &str, update: bool) -> Self {
        SnapshotTester {
            snapshot_dir: Path::new("tests/snapshots").join(dir),
            update_snapshots: update,
        }
    }

    pub fn assert_snapshot(&self, name: &str, actual: &str) -> Result<(), String> {
        fs::create_dir_all(&self.snapshot_dir).map_err(|e| e.to_string())?;

        let path = self.snapshot_dir.join(format!("{}.snap", name));

        if self.update_snapshots || !path.exists() {
            fs::write(&path, actual).map_err(|e| e.to_string())?;
            return Ok(());
        }

        let expected = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if actual == expected {
            Ok(())
        } else {
            Err(format!(
                "Snapshot '{}' berbeda!\n\
                 Expected:\n{}\n\
                 Actual:\n{}\n\
                 Jalankan `rakit test --update` untuk update snapshot.",
                name, expected, actual
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_create_and_compare() {
        let dir = "test_snapshots";
        let tester = SnapshotTester::with_update(dir, true);
        let result = tester.assert_snapshot("test1", "output");
        assert!(result.is_ok());
    }
}
