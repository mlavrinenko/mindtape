//! The filesystem side of `at.toml`: `~` expansion, one-level glob expansion,
//! and the project and fork tests, all through the injected [`AtFs`].

use std::path::{Component, Path, PathBuf};

use globset::Glob;

use super::AtError;

/// The few filesystem reads resolution needs, injected so tests touch no disk.
pub trait AtFs {
    /// The names of the directories directly under `dir`, symlinks to one
    /// included; empty when it can't be read.
    fn subdirs(&self, dir: &Path) -> Vec<String>;
    /// Whether `path` is a directory.
    fn is_dir(&self, path: &Path) -> bool;
    /// Whether `path` is a file.
    fn is_file(&self, path: &Path) -> bool;
    /// The text of the file at `path`, if it reads.
    fn read_file(&self, path: &Path) -> Option<String>;
}

/// The real filesystem.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiskFs;

impl AtFs for DiskFs {
    fn subdirs(&self, dir: &Path) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        names.sort();
        names
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read_file(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
}

/// Expand a leading `~` in `path` against `home`; other paths pass through.
pub(super) fn expand_tilde(path: &str, home: Option<&Path>) -> PathBuf {
    let path = Path::new(path);
    match (home, path.strip_prefix("~")) {
        (Some(home), Ok(rest)) => home.join(rest),
        _ => path.to_path_buf(),
    }
}

/// Whether a pattern holds a glob character, so it names a set, not a path.
pub(super) fn is_glob(pattern: &str) -> bool {
    pattern.contains(['*', '?', '[', '{'])
}

/// The directories under `path` that hold a project, where each glob segment
/// reads one directory level. Sorted by path.
pub(super) fn expand_glob(path: &Path, fs: &dyn AtFs) -> Result<Vec<PathBuf>, AtError> {
    let mut found = vec![PathBuf::new()];
    for component in path.components() {
        let segment = component.as_os_str().to_string_lossy();
        found = match component {
            Component::Normal(_) if is_glob(&segment) => {
                let matcher = Glob::new(&segment)
                    .map_err(|source| AtError::Glob {
                        pattern: path.display().to_string(),
                        source,
                    })?
                    .compile_matcher();
                found
                    .iter()
                    .flat_map(|dir| {
                        fs.subdirs(dir)
                            .into_iter()
                            .filter(|name| matcher.is_match(name))
                            .map(|name| dir.join(name))
                    })
                    .collect()
            }
            _ => found.into_iter().map(|dir| dir.join(component)).collect(),
        };
    }
    found.retain(|dir| is_project(dir, fs));
    found.sort();
    Ok(found)
}

/// Whether `dir` holds a `.mindtape/` directory.
pub(super) fn is_project(dir: &Path, fs: &dyn AtFs) -> bool {
    fs.is_dir(&dir.join(".mindtape"))
}

/// Whether `dir` is a linked git worktree: its `.git` file points at a gitdir
/// that holds `commondir`, which a submodule's gitdir lacks.
pub(super) fn is_fork(dir: &Path, fs: &dyn AtFs) -> bool {
    let Some(text) = fs.read_file(&dir.join(".git")) else {
        return false;
    };
    let Some(gitdir) = text.trim().strip_prefix("gitdir:") else {
        return false;
    };
    fs.is_file(&dir.join(gitdir.trim()).join("commondir"))
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod tests;
