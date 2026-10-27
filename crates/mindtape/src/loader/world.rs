//! The disk-backed Typst world every task file is evaluated in.

use std::path::Path;

use typst_harvest::HarvestWorld;
use typst_world::{DiskProvider, WorldError, root_file_id};

use crate::model::slash_path;

/// A world that evaluates `file` with `root` as the project root.
///
/// `HarvestWorld::with_root` would do the same, but it names the main file
/// by the platform's relative path, and Typst refuses a virtual path holding
/// `\`. So on Windows every file in a folder, such as `tasks\a.typ`, failed
/// with "bad virtual path". This builds the main file's id from the
/// `/`-separated path instead.
///
/// # Errors
/// Returns an error if `file` or `root` cannot be canonicalized, or `file`
/// is not under `root`.
pub fn disk_world(file: &Path, root: &Path) -> Result<HarvestWorld, WorldError> {
    let canon = |path: &Path| {
        path.canonicalize()
            .map_err(|err| WorldError::World(format!("canonicalize {}: {err}", path.display())))
    };
    let (abs, root) = (canon(file)?, canon(root)?);
    let rel = abs.strip_prefix(&root).map_err(|_| {
        WorldError::World(format!(
            "file {} is not under project root {}",
            abs.display(),
            root.display()
        ))
    })?;
    let id = root_file_id(&slash_path(rel))?;
    Ok(HarvestWorld::with_provider(
        id,
        root,
        Box::new(DiskProvider),
    ))
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;
