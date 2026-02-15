//! File watcher for `MindTape`.
//!
//! Watches configured directories for `.typ` file changes, re-indexes
//! modified files, and removes deleted files from the store.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::WalkBuilder;
use log::{debug, info, trace, warn};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use thiserror::Error;

use crate::config::WatchEntry;
use crate::store::{self, SqliteStore, Store, StoreError};
use crate::world::{self, MindTapeWorld};

#[derive(Debug, Error)]
pub enum WatchError {
    #[error("notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("store error: {0}")]
    Store(#[from] StoreError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("watch error: {0}")]
    Other(String),
}

/// Result of an initial directory scan.
#[derive(Debug, Default)]
pub struct ScanResult {
    pub found: usize,
    pub indexed: usize,
    pub skipped: usize,
    pub errors: usize,
}

/// Resolved watch entry with absolute path and project root.
struct ResolvedEntry {
    path: PathBuf,
    project_root: PathBuf,
    recursive: bool,
}

pub struct Watcher {
    store: SqliteStore,
    entries: Vec<ResolvedEntry>,
}

impl Watcher {
    /// Create a new watcher from a store and config entries.
    ///
    /// Resolves each entry's path to absolute, discovers the project root,
    /// and loads `.mindtapeignore` patterns.
    ///
    /// # Errors
    ///
    /// Returns [`WatchError::Other`] if a watch path cannot be resolved
    /// (e.g. the directory does not exist), or [`WatchError::Io`] if
    /// canonicalization fails.
    pub fn new(store: SqliteStore, entries: &[WatchEntry]) -> Result<Self, WatchError> {
        let mut resolved = Vec::with_capacity(entries.len());
        for entry in entries {
            let path = crate::config::expand_tilde(&entry.path);
            let path = std::fs::canonicalize(&path).map_err(|err| {
                WatchError::Other(format!(
                    "cannot resolve watch path {}: {err}",
                    entry.path
                ))
            })?;
            let project_root = world::find_project_root(&path);
            resolved.push(ResolvedEntry {
                path,
                project_root,
                recursive: entry.recursive,
            });
        }
        Ok(Self {
            store,
            entries: resolved,
        })
    }

    /// Walk all watched directories and index every `.typ` file found.
    pub fn initial_scan(&mut self) -> ScanResult {
        let mut result = ScanResult::default();

        // Collect files first to avoid borrowing self immutably (entries)
        // and mutably (index_one) at the same time.
        let mut files: Vec<(PathBuf, PathBuf)> = Vec::new();
        for entry in &self.entries {
            let walker = WalkBuilder::new(&entry.path)
                .hidden(false)
                .add_custom_ignore_filename(".mindtapeignore")
                .build();

            for dir_entry in walker.flatten() {
                let path = dir_entry.into_path();
                if !is_typ_file(&path) {
                    continue;
                }
                files.push((path, entry.project_root.clone()));
            }
        }

        result.found = files.len();
        for (path, project_root) in &files {
            match self.index_one(path, project_root) {
                Ok(true) => result.indexed += 1,
                Ok(false) => result.skipped += 1,
                Err(err) => {
                    warn!("error indexing {}: {err}", path.display());
                    result.errors += 1;
                }
            }
        }

        result
    }

    /// Handle a single debounced file event.
    ///
    /// If the file exists and is a `.typ` file that passes ignore checks,
    /// re-index it. If it no longer exists, remove it from the store.
    pub fn handle_event(&mut self, path: &Path) {
        // Extract what we need from the entry before any mutable borrows.
        let (project_root, is_ignored) = {
            let Some(entry) = self.find_entry(path) else {
                return;
            };
            let ignore = build_ignore(&entry.path, path);
            let ignored = ignore
                .matched_path_or_any_parents(path, false)
                .is_ignore();
            (entry.project_root.clone(), ignored)
        };

        // Only care about .typ files in both branches.
        if path.extension().is_none_or(|ext| ext != "typ") {
            return;
        }

        if path.exists() {
            if !path.is_file() || is_ignored {
                return;
            }
            match self.index_one(path, &project_root) {
                Ok(true) => info!("indexed {}", path.display()),
                Ok(false) => debug!("unchanged {}", path.display()),
                Err(err) => warn!("error indexing {}: {err}", path.display()),
            }
        } else {
            // File was deleted — remove from store.
            let Ok(rel) = path.strip_prefix(&project_root) else {
                return;
            };
            match self.store.remove_task_file(rel) {
                Ok(()) => info!("removed {}", path.display()),
                Err(err) => warn!("error removing {}: {err}", path.display()),
            }
        }
    }

    /// Start the file watcher event loop (blocks forever).
    ///
    /// Filters out `Access` events (e.g. file opens) to prevent infinite
    /// re-indexing loops caused by our own reads triggering inotify.
    /// Applies a simple per-path debounce window to coalesce rapid edits.
    ///
    /// # Errors
    ///
    /// Returns [`WatchError::Notify`] if the watcher cannot be created
    /// or a watched path cannot be registered.
    pub fn run(mut self) -> Result<(), WatchError> {
        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher: RecommendedWatcher =
            NotifyWatcher::new(tx, notify::Config::default())?;

        for entry in &self.entries {
            let mode = if entry.recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            watcher.watch(&entry.path, mode)?;
            info!("watching {}", entry.path.display());
        }

        let debounce = Duration::from_millis(300);
        let mut last_seen: HashMap<PathBuf, Instant> = HashMap::new();

        for result in rx {
            match result {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
                        continue;
                    }
                    let now = Instant::now();
                    for path in event.paths {
                        if is_inside_dotgit(&path) {
                            continue;
                        }
                        if let Some(&last) = last_seen.get(&path) {
                            if now.duration_since(last) < debounce {
                                trace!("debounced {}", path.display());
                                continue;
                            }
                        }
                        last_seen.insert(path.clone(), now);
                        self.handle_event(&path);
                    }
                }
                Err(err) => {
                    warn!("watch error: {err}");
                }
            }
        }

        // Keep watcher alive for the duration of the loop.
        drop(watcher);
        Ok(())
    }

    /// Index a single file using the existing store pipeline.
    fn index_one(&mut self, path: &Path, project_root: &Path) -> Result<bool, StoreError> {
        let world = MindTapeWorld::new(path)?;
        store::index_file_with_deps(&mut self.store, &world, path, project_root)
    }

    /// Find which resolved entry contains the given path.
    fn find_entry(&self, path: &Path) -> Option<&ResolvedEntry> {
        self.entries
            .iter()
            .find(|entry| path.starts_with(&entry.path))
    }
}

/// Build a `Gitignore` matcher for a file path, collecting ignore rules from
/// the watch root down to the file's parent directory. Called on each event
/// so that changes to ignore files are picked up without restarting.
///
/// Walks from `root` down to the file's parent, loading `.gitignore` and
/// `.mindtapeignore` at each level (deeper files override shallower ones).
/// Global gitignore has the lowest precedence.
fn build_ignore(root: &Path, file: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(root);

    // Global gitignore (lowest precedence).
    if let Some(global) = global_gitignore_path() {
        if global.exists() {
            builder.add(&global);
        }
    }

    // Collect directories from root down to the file's parent.
    let target = file.parent().unwrap_or(root);
    let mut dirs_to_check: Vec<&Path> = Vec::new();
    let mut current = target;
    loop {
        dirs_to_check.push(current);
        if current == root {
            break;
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent,
            _ => break,
        }
    }
    // Reverse so we go from root (lower precedence) to deepest dir (higher).
    dirs_to_check.reverse();

    for dir in dirs_to_check {
        let gitignore = dir.join(".gitignore");
        if gitignore.exists() {
            builder.add(&gitignore);
        }
        let mindtapeignore = dir.join(".mindtapeignore");
        if mindtapeignore.exists() {
            builder.add(&mindtapeignore);
        }
    }

    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

/// Find the global gitignore file path.
///
/// Checks `git config --global core.excludesFile` first, then falls back
/// to the XDG-compliant default (`$XDG_CONFIG_HOME/git/ignore` or
/// `~/.config/git/ignore`).
fn global_gitignore_path() -> Option<PathBuf> {
    // Try git config first.
    if let Ok(output) = std::process::Command::new("git")
        .args(["config", "--global", "core.excludesFile"])
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(crate::config::expand_tilde(&path));
            }
        }
    }

    // XDG fallback.
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("git/ignore"));
    }

    directories::BaseDirs::new().map(|d| d.config_dir().join("git/ignore"))
}

fn is_typ_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "typ") && path.is_file()
}

/// Check if a path is inside a `.git` directory.
fn is_inside_dotgit(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str() == ".git")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
#[path = "watcher_tests.rs"]
mod tests;
