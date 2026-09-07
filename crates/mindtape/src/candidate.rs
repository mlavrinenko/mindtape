//! Validate a rendered task *before* it is written
//! (`truth/tasks/mt-add-writes-an-invalid-task-before-it-validates-one.typ`).
//!
//! [`load_project`](crate::load_project) answers "what do the files on disk
//! say"; this module answers "what would they say if this one more file
//! landed here", without landing it. A writer (`mt add`) renders into a
//! buffer, asks [`check_candidate`], and only then commits the buffer to its
//! path — so a rejected write leaves nothing behind to repair.
//!
//! The candidate is evaluated exactly where it would live: its
//! [`Candidate::rel`] is the file id Typst resolves relative imports against,
//! and the folder-derived tags and nesting depth are read off that same path.
//! What makes this possible without a file is
//! [`World::with_source_override`](typst_world::World::with_source_override) —
//! every disk-backed constructor canonicalizes its path, so the world is built
//! from the id directly instead.

use std::path::Path;

use typst_harvest::{HVal, HarvestWorld, harvest};
use typst_world::{DiskProvider, root_file_id};

use crate::config::Config;
use crate::loader::LoadError;
use crate::model::Task;
use crate::validate::Violation;

/// A task source that does not exist on disk, and the project-relative path it
/// would be written to.
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'a> {
    /// Destination path, relative to the project root.
    pub rel: &'a Path,
    /// The rendered task source, exactly as it would be written.
    pub source: &'a str,
}

/// Validate `candidate` against `existing` as if it had already been written to
/// [`Candidate::rel`], returning only the violations that name the candidate
/// itself.
///
/// `existing` is the project's already-loaded tasks — no second load happens
/// here — with any task already at `rel` replaced, so a rewrite is checked as a
/// rewrite rather than as a duplicate id.
///
/// # Errors
/// Returns a [`LoadError`] when the source fails to evaluate or harvests no
/// `mindtape.task` marker. Both are failures of the source, not of the project.
pub fn check_candidate(
    root: &Path,
    config: &Config,
    existing: &[Task],
    candidate: Candidate<'_>,
) -> Result<Vec<Violation>, LoadError> {
    let task = harvest_candidate(root, config, candidate)?;
    let id = task.id.clone();
    let mut tasks: Vec<Task> = existing
        .iter()
        .filter(|task| task.file != candidate.rel)
        .cloned()
        .collect();
    tasks.push(task);
    tasks.sort_by(|left, right| left.id.cmp(&right.id));
    crate::dep_rank::annotate(&mut tasks, config);
    Ok(crate::validate::validate(&tasks, config)
        .into_iter()
        .filter(|violation| violation.task_id == id)
        .collect())
}

/// Evaluate `candidate`'s source at its destination path and build the [`Task`]
/// it would load as — identity from the stem, folder-derived tags and depth
/// from the path, body from the source.
///
/// # Errors
/// Returns a [`LoadError`] when the world cannot be built, evaluation fails, or
/// no `mindtape.task` marker is emitted.
pub fn harvest_candidate(
    root: &Path,
    config: &Config,
    candidate: Candidate<'_>,
) -> Result<Task, LoadError> {
    let marker = eval_marker(root, config, candidate)?;
    Ok(crate::loader::task_at(
        root,
        config,
        candidate.rel,
        &marker,
        candidate.source,
    ))
}

/// Evaluate the candidate source and return its first `mindtape.task` marker.
fn eval_marker(root: &Path, config: &Config, candidate: Candidate<'_>) -> Result<HVal, LoadError> {
    let fail = |message: String| LoadError {
        file: candidate.rel.to_path_buf(),
        message,
    };
    // Typst virtual paths are `/`-separated on every platform, so a Windows
    // `tasks\foo.typ` has to be spelled the way the loader's own ids are.
    let vpath = candidate.rel.to_string_lossy().replace('\\', "/");
    let id = root_file_id(&vpath).map_err(|err| fail(err.to_string()))?;
    let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    // `with_package_root` installs the provider that resolves `@local/...`,
    // replacing the `DiskProvider` this constructor requires; the same pair
    // `HarvestWorld::with_root` sets up for a file that exists.
    let mut world = HarvestWorld::with_provider(id, root_abs, Box::new(DiskProvider))
        .with_package_root(crate::loader::package_root());
    for (name, dir) in crate::loader::package_overrides(root, config) {
        world = world.with_local_package(&name, dir);
    }
    let world = world.with_source_override(id, candidate.source);

    let harvested = harvest(&world).map_err(|err| fail(err.to_string()))?;
    harvested
        .with_marker("mindtape.task")
        .next()
        .map(|found| found.value.clone())
        .ok_or_else(|| fail("source harvested no task marker".to_owned()))
}

#[cfg(test)]
#[path = "candidate_tests.rs"]
mod tests;
