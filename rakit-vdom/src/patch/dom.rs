use crate::diff::DiffResult;

pub struct DomPatchApplicator;

impl DomPatchApplicator {
    pub fn new() -> Self {
        DomPatchApplicator
    }

    pub fn apply_patches(&self, _result: &DiffResult) {
    }
}
