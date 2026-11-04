//! Which files `mt fix` scans: the configured task roots and project-owned
//! template dirs, or the whole project tree with `--all`.
//!
//! Default scope follows `tasks.roots` and `[templates] dirs`. A template dir
//! a pack appended outside the project root is skipped, because it is not the
//! project's file to edit. `--all` starts at the project root instead and
//! skips `.git`, `target` and `.mindtape/cache`.
//!
//! A symlinked directory is never followed in either mode: a cycle or a shared
//! tree would make the scan unbounded. A symlinked file is collected so the
//! fixer can read it and report that it will not rewrite one.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mindtape::Config;

/// Every `.typ` file the scope holds, absolute and sorted.
pub(super) fn collect(root: &Path, config: &Config, all: bool) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();
    let dirs = if all {
        vec![root.to_path_buf()]
    } else {
        default_dirs(root, config)
    };
    for dir in dirs {
        walk(&dir, &mut files);
    }
    files.into_iter().collect()
}

/// `tasks.roots` plus each `[templates] dirs` entry that stays under `root`.
fn default_dirs(root: &Path, config: &Config) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = config
        .tasks
        .roots
        .iter()
        .map(|spec| resolve(root, spec.path()))
        .collect();
    dirs.extend(
        config
            .templates
            .dirs
            .iter()
            .map(|dir| resolve(root, dir))
            .filter(|dir| dir.starts_with(root)),
    );
    dirs
}

/// `dir` against `root`. An absolute `dir` replaces `root`, as `Path::join`
/// does. An existing path is canonical, so `.` and `..` compare by the folder
/// they name; a missing one is left as spelled, since the walk finds nothing
/// under it anyway.
fn resolve(root: &Path, dir: &str) -> PathBuf {
    let path = Path::new(dir);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    mindtape::canonical(&joined).unwrap_or(joined)
}

/// Add every `.typ` file under `dir` to `files`.
fn walk(dir: &Path, files: &mut BTreeSet<PathBuf>) {
    if !dir.is_dir() {
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

#[cfg(test)]
#[path = "scope_tests.rs"]
mod tests;
