//! The scratch directory a fetch builds its tree in, inside the cache root,
//! and the rename that lands the tree. A scratch directory a killed fetch
//! left behind is swept by the next fetch once it is an hour old, since
//! `Drop` doesn't run on Ctrl-C.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::config::pack_sources::{FetchError, hash};

/// How every scratch directory's name starts.
const PREFIX: &str = ".fetch-";

/// How old a scratch directory must be before a fetch removes it: longer
/// than any fetch runs.
pub(super) const STALE_AFTER: Duration = Duration::from_secs(3600);

/// A temporary directory inside the cache root, removed when dropped.
pub(super) struct Scratch(pub(super) PathBuf);

impl Scratch {
    pub(super) fn new(cache_root: &Path) -> Result<Self, FetchError> {
        let dir = cache_root.join(format!("{PREFIX}{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir(&dir)?;
        Ok(Self(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Remove every scratch directory under `cache_root` last modified at
/// least `older_than` ago. One whose time lies ahead counts as new.
pub(super) fn sweep(cache_root: &Path, older_than: Duration) {
    let Ok(entries) = std::fs::read_dir(cache_root) else {
        return;
    };
    for entry in entries.flatten() {
        let scratch = entry.file_name().to_string_lossy().starts_with(PREFIX);
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let age = meta
            .modified()
            .ok()
            .and_then(|modified| modified.elapsed().ok())
            .unwrap_or(Duration::ZERO);
        if scratch && meta.is_dir() && age >= older_than {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// Rename `tree` to `target`. When another fetch got there first, its tree
/// must hash to `actual` too.
pub(super) fn settle(tree: &Path, target: &Path, actual: &str) -> Result<(), FetchError> {
    let Err(error) = std::fs::rename(tree, target) else {
        return Ok(());
    };
    if !target.is_dir() {
        return Err(error.into());
    }
    if hash(target)? == actual {
        return Ok(());
    }
    Err(FetchError::Occupied {
        dir: target.to_path_buf(),
    })
}

#[cfg(test)]
#[path = "scratch_tests.rs"]
mod tests;
