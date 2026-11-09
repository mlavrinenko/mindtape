//! Which files `mt fix` scans: the working-tree view task discovery shares,
//! plus the project's template dirs, or every project `.typ` with `--all`.
//!
//! The default scope is [`mindtape::visible_task_sources`]: every configured
//! task root's tracked files and untracked files Git does not ignore, which
//! excludes untracked ignored files and symlinks. `--all` is
//! [`mindtape::visible_typ_files`]: every Git-visible `.typ` in the project,
//! non-task ones such as `lib/` and docs included. A project outside a Git
//! work tree walks the filesystem instead, keeping the legacy `.git`,
//! `target` and `.mindtape/cache` skips.
//!
//! Both scopes add every configured template dir that stays under `root`,
//! including the dirs a directory pack contributes after the config merge; a
//! pack dir outside the project is skipped, because it is not the project's
//! file to edit.
//!
//! A symlinked directory is never followed in either mode: a cycle or a shared
//! tree would make the scan unbounded. A symlinked file under `--all` is
//! collected so the fixer can read it and report that it will not rewrite one.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mindtape::{Config, TaskSource};

use crate::diag::{Diag, Kind, Result};

/// Every `.typ` file the scope holds, absolute and sorted.
///
/// # Errors
/// A Git failure listing the project or one of its task roots.
pub(super) fn collect(root: &Path, config: &Config, all: bool) -> Result<Vec<PathBuf>> {
    let mut files = if all {
        project_files(root)?
    } else {
        mindtape::visible_task_sources(root, config)
            .map_err(listing_diag)?
            .into_iter()
            .collect()
    };
    collect_templates(root, config, &mut files);
    Ok(files.into_iter().collect())
}

/// The `--all` listing: every Git-visible `.typ`, else the non-Git walk with
/// the legacy `target` skip applied.
fn project_files(root: &Path) -> Result<BTreeSet<PathBuf>> {
    let files = mindtape::visible_typ_files(root).map_err(listing_diag)?;
    if mindtape::project_is_worktree(root) {
        return Ok(files.into_iter().collect());
    }
    Ok((files.into_iter())
        .filter(|path| !under_target(root, path))
        .collect())
}

/// Whether `path` sits under a `target` directory below `root`: the one
/// legacy `--all` skip the shared fallback walk does not already apply.
fn under_target(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root)
        .is_ok_and(|rel| (rel.components()).any(|component| component.as_os_str() == "target"))
}

/// Add every `.typ` file under each configured template dir that stays under
/// `root`.
fn collect_templates(root: &Path, config: &Config, files: &mut BTreeSet<PathBuf>) {
    let source = TaskSource::new(root, config);
    for dir in source.templates() {
        if dir.starts_with(source.root()) {
            walk(dir, files);
        }
    }
}

/// Add every `.typ` file under `dir` to `files`. A symlinked `dir` is not
/// entered, and a symlinked file is added only when it does not point at a
/// directory.
fn walk(dir: &Path, files: &mut BTreeSet<PathBuf>) {
    if !std::fs::symlink_metadata(dir).is_ok_and(|meta| meta.is_dir()) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            if !path.is_dir() && is_typ(&path) {
                files.insert(path);
            }
            continue;
        }
        if kind.is_dir() {
            if !skipped(&path) {
                walk(&path, files);
            }
        } else if kind.is_file() && is_typ(&path) {
            files.insert(path);
        }
    }
}

/// Whether `dir` is one of the trees a scan never enters.
fn skipped(dir: &Path) -> bool {
    let name = dir.file_name().and_then(|name| name.to_str());
    if matches!(name, Some(".git" | "target")) {
        return true;
    }
    name == Some("cache")
        && dir
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            == Some(".mindtape")
}

/// Whether `path` names a Typst file by extension.
fn is_typ(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("typ")
}

/// The diagnostic a failed listing contributes.
fn listing_diag(error: mindtape::LoadError) -> Diag {
    crate::warnings::at_place(
        Diag::verbatim(Kind::Failed, error.message),
        &error.file,
        error.line,
        error.column,
    )
    .typst_hints(error.hints)
}

#[cfg(test)]
#[path = "scope_tests.rs"]
mod tests;
