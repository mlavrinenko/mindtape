//! The one destination guard every task writer asks before it creates
//! anything: `mt add`, `mt import` and `mt recur` (spawn, mutate, skip and
//! advance).
//!
//! The question is the loader's own ([`mindtape::would_be_visible`]): would a
//! task written here be loaded back? A reserved path (`.mindtape`, `.git`, a
//! `[templates] dirs` entry) and a new file under an ignore rule are answered
//! "no", and the write is refused before a directory or file exists. A tracked
//! file under an ignore rule is still visible, so mutating or skipping one is
//! allowed while a new sibling beside it is not.
//!
//! The path is resolved first, against the filesystem, so a symlinked
//! directory or a `..` after one cannot make the policy inspect a lexical path
//! other than the one the write would reach: `.` and `..` are folded and every
//! symlink on the nearest existing ancestor is followed, with the components
//! below it appended. Only then does [`mindtape::would_be_visible`] answer.
//!
//! A destination outside every root is not this guard's to refuse: it stays
//! the warning [`crate::cmd::add::warn_if_outside_roots`] has always printed.

use std::path::{Path, PathBuf};

use crate::diag::{Diag, Hint, Result};
use crate::util::Project;
use mindtape::NotVisible;

/// Which command is writing, for the fix its refusal names.
#[derive(Clone, Copy)]
pub(crate) enum Mode {
    /// `mt add`: another `--into`/`--to`.
    Add,
    /// `mt import`: another `--into`/`--to`.
    Import,
    /// `mt recur`: move the task, or unignore its path.
    Recur,
}

impl Mode {
    /// The hint a refusal of this mode carries.
    fn hint(self) -> Hint {
        match self {
            Self::Add | Self::Import => Hint::new("choose another --into/--to"),
            Self::Recur => Hint::new("move the task, or unignore its path"),
        }
    }
}

/// Refuse `path` when a task written there would be invisible to the loader.
///
/// Nothing is created here, so a preview can ask the same question a real run
/// does. The refusal names the resolved final path and the reason
/// [`mindtape::would_be_visible`] gave, with the fix `mode` reads.
///
/// # Errors
/// A [`Diag`] when the resolved destination is reserved, a symlink, not a
/// `.typ` file, a directory, ignored by Git, or a path Git could not answer
/// for. A destination outside every configured root is accepted.
pub(crate) fn check(project: &Project, path: &Path, mode: Mode) -> Result<()> {
    let candidate = mindtape::TaskPath::new(&project.root, &resolve(path));
    let resolved = candidate.abs();
    match mindtape::would_be_visible(&project.root, &project.config, resolved) {
        Ok(()) | Err(NotVisible::OutsideRoots) => Ok(()),
        Err(reason) => Err(Diag::refused("destination would be invisible")
            .value(mindtape::display_path(resolved))
            .cause(&reason)
            .hint(mode.hint())),
    }
}

/// `path` as the filesystem would reach it: its parent canonicalized (following
/// the parent's symlinks and folding its `.`/`..`), then the final component
/// appended without following it.
///
/// The final component stays as written so the policy can see that it is a
/// symlink and refuse it. Following it here would answer for the target, and a
/// write through the link would leave a task source the loader never lists.
///
/// A parent whose nearest existing ancestor cannot be resolved keeps its
/// absolute lexical form, so the policy still answers rather than the guard
/// failing on an unrelated filesystem error.
pub(crate) fn resolve(path: &Path) -> PathBuf {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let (Some(name), Some(parent)) = (absolute.file_name(), absolute.parent()) else {
        return absolute;
    };
    let mut resolved = resolve_dir(parent);
    resolved.push(name);
    resolved
}

/// `dir` as the filesystem reaches it: the nearest existing ancestor
/// canonicalized, then the components below it appended.
fn resolve_dir(dir: &Path) -> PathBuf {
    let mut appended: Vec<std::ffi::OsString> = Vec::new();
    let mut cursor = dir;
    loop {
        if let Ok(mut resolved) = mindtape::canonical(cursor) {
            for component in appended.iter().rev() {
                resolved.push(component);
            }
            return resolved;
        }
        let (Some(name), Some(parent)) = (cursor.file_name(), cursor.parent()) else {
            return dir.to_path_buf();
        };
        appended.push(name.to_os_string());
        cursor = parent;
    }
}

#[cfg(test)]
#[path = "write_guard_tests.rs"]
mod tests;
