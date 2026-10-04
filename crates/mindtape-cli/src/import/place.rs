//! Where each imported record lands, and which stems are already taken.
//!
//! With neither `--into` nor `--to`, a record lands in the folder its `file`
//! names when that folder lies under a configured root, so an export of a
//! tree imports back as the same tree. Either flag sends the whole batch to
//! one destination, and a record whose `file` is empty, absolute, climbs out
//! with `..` or leaves every root lands in that destination too.
//!
//! A stem is taken when any loaded task, or any `.typ` file in a destination,
//! already uses it, so `mt import` suffixes it (`-2`) instead of writing a
//! second task with the same identity.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use mindtape::CanonicalTask;

use super::ImportArgs;
use crate::add::dest_dir;
use crate::util::Project;

/// The destinations of one batch and the stems claimed so far.
pub(super) struct Placement<'a> {
    project: &'a Project,
    fallback: PathBuf,
    by_file: bool,
    taken: HashSet<String>,
    scanned: HashSet<PathBuf>,
}

impl<'a> Placement<'a> {
    /// The placement for `args` in `project`: every loaded task's stem is
    /// taken from the start.
    pub(super) fn new(project: &'a Project, args: &ImportArgs) -> Result<Self> {
        let fallback = dest_dir(
            args.into.as_deref().unwrap_or("."),
            args.to.as_deref(),
            project,
        )?;
        let taken = project
            .outcome
            .tasks
            .iter()
            .filter_map(|task| task.file.file_stem())
            .map(|stem| stem.to_string_lossy().into_owned())
            .collect();
        Ok(Self {
            project,
            fallback,
            by_file: args.into.is_none() && args.to.is_none(),
            taken,
            scanned: HashSet::new(),
        })
    }

    /// The folder `record` lands in, with its on-disk stems marked taken.
    pub(super) fn dest_for(&mut self, record: &CanonicalTask) -> PathBuf {
        let dest = self
            .record_dir(record)
            .unwrap_or_else(|| self.fallback.clone());
        if self.scanned.insert(dest.clone()) {
            self.taken.extend(existing_stems(&dest));
        }
        dest
    }

    /// The stems claimed so far, for the collision check; a claimed stem is
    /// added by the caller.
    pub(super) fn taken(&mut self) -> &mut HashSet<String> {
        &mut self.taken
    }

    fn record_dir(&self, record: &CanonicalTask) -> Option<PathBuf> {
        if !self.by_file {
            return None;
        }
        let dir = Path::new(&record.file).parent()?;
        let relative = !dir.as_os_str().is_empty()
            && dir
                .components()
                .all(|part| matches!(part, Component::Normal(_)));
        let dir = self.project.root.join(dir);
        let under_a_root = self
            .project
            .config
            .tasks
            .roots
            .iter()
            .any(|root| dir.starts_with(self.project.root.join(root.path())));
        (relative && under_a_root).then_some(dir)
    }
}

/// Filename stems of the `.typ` files directly under `dest`.
pub(super) fn existing_stems(dest: &Path) -> HashSet<String> {
    let Ok(entries) = std::fs::read_dir(dest) else {
        return HashSet::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("typ"))
        .filter_map(|path| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect()
}
