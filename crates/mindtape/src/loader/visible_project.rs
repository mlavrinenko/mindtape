//! The project-wide `.typ` listing `mt fix --all` reads.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::query::GitFailure;
use crate::task_source::has_reserved_component;

use super::LoadError;
use super::visible_git::{RepoListing, repo_of};

/// Every `.typ` file the project's Git working tree exposes, absolute and
/// sorted, for `mt fix --all`.
///
/// This is not a task-root listing: it includes non-task `.typ` files such as
/// `lib/` and docs. It includes tracked files even where an ignore rule
/// matches, and untracked files Git does not ignore. It excludes untracked
/// ignored files, tracked-deleted files, symlinks that point at a directory,
/// and any path with a `.mindtape` or `.git` component (so configured template
/// dirs under `.mindtape` are the caller's separate scope, not this one).
///
/// A project outside a work tree, or a machine without `git`, walks the
/// project root with the same reserved-name skips and no directory symlinks.
///
/// # Errors
/// A Git failure other than a missing `git` or a non-worktree project root.
pub fn visible_typ_files(root: &Path) -> Result<Vec<PathBuf>, LoadError> {
    match project_git_files(root) {
        Ok(Some(files)) => Ok(files.into_iter().collect()),
        Ok(None) => Ok(walk_project_typ(root)),
        Err(failure) => Err(project_error(root, &failure)),
    }
}

/// Whether `root` is inside a Git work tree: the mode [`visible_typ_files`]
/// lists in. A missing `git`, a directory outside every work tree, or a Git
/// failure answers `false`, the fallback walk's mode.
#[must_use]
pub fn project_is_worktree(root: &Path) -> bool {
    matches!(repo_of(root), Ok(Some(_)))
}

/// The project's Git-visible `.typ` files, or `None` to fall back.
fn project_git_files(root: &Path) -> Result<Option<BTreeSet<PathBuf>>, GitFailure> {
    let Some(repo) = repo_of(root)? else {
        return Ok(None);
    };
    let listing = match RepoListing::read(&repo.top) {
        Ok(listing) => listing,
        Err(failure) if failure.permits_fallback() => return Ok(None),
        Err(failure) => return Err(failure),
    };
    let mut files = BTreeSet::new();
    for path in listing.files {
        let Ok(rel) = path.strip_prefix(&repo.prefix) else {
            continue;
        };
        let abs = crate::config::roots::normalize_path(&root.join(rel));
        if !has_reserved_component(rel) && is_project_typ(&abs) {
            files.insert(abs);
        }
    }
    Ok(Some(files))
}

/// Whether `path` is a `.typ` file `mt fix --all` may read: a regular file, or
/// a symlink that does not point at a directory. A tracked-deleted path has no
/// entry and is excluded.
fn is_project_typ(path: &Path) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    let kind = metadata.file_type();
    if kind.is_dir() || path.extension().and_then(OsStr::to_str) != Some("typ") {
        return false;
    }
    if kind.is_symlink() {
        !path.is_dir()
    } else {
        kind.is_file()
    }
}

/// The non-Git `mt fix --all` walk: every `.typ` file under `root`, skipping
/// `.mindtape` and `.git` components and never following a directory symlink.
fn walk_project_typ(root: &Path) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();
    collect_project_typ(root, &mut files);
    files.into_iter().collect()
}

fn collect_project_typ(dir: &Path, out: &mut BTreeSet<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if has_reserved_component(Path::new(&entry.file_name())) {
            continue;
        }
        if kind.is_symlink() {
            if is_project_typ(&path) {
                out.insert(path);
            }
        } else if kind.is_dir() {
            collect_project_typ(&path, out);
        } else if kind.is_file() && path.extension().and_then(OsStr::to_str) == Some("typ") {
            out.insert(path);
        }
    }
}

/// The load error a failed project-wide listing contributes.
fn project_error(root: &Path, failure: &GitFailure) -> LoadError {
    LoadError {
        file: PathBuf::from("."),
        message: format!(
            "can't list the project root `{}` with git: {}",
            crate::display_path(root),
            failure.reason()
        ),
        ..LoadError::default()
    }
}
