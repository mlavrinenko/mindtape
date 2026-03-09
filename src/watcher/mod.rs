//! File watcher for `MindTape`.
//!
//! Watches configured directories for `.typ` file changes, re-indexes
//! modified files, and removes deleted files from the store.

mod ignore;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ::ignore::WalkBuilder;
use log::{debug, info, trace, warn};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use thiserror::Error;

use crate::config::{self, WatchEntry};
use crate::store::{self, SqliteStore, Store, StoreError};
use crate::world::MindTapeWorld;

use self::ignore::build_ignore;

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

/// Resolved watch entry with absolute path.
struct ResolvedEntry {
    path: PathBuf,
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
            resolved.push(resolve_entry(entry)?);
        }
        Ok(Self {
            store,
            entries: resolved,
        })
    }

    /// Walk all watched directories and index every `.typ` file found.
    pub fn initial_scan(&mut self) -> ScanResult {
        let watch_roots: Vec<_> = self.entries.iter().map(|e| e.path.clone()).collect();
        self.scan_entries(&watch_roots)
    }

    /// Scan specific directories and index their `.typ` files.
    fn scan_entries(&mut self, watch_roots: &[PathBuf]) -> ScanResult {
        let mut result = ScanResult::default();

        let mut files: Vec<(PathBuf, PathBuf)> = Vec::new();
        for watch_root in watch_roots {
            let walker = WalkBuilder::new(watch_root)
                .hidden(false)
                .add_custom_ignore_filename(".mindtapeignore")
                .build();

            for dir_entry in walker.flatten() {
                let file_path = dir_entry.into_path();
                if !is_typ_file(&file_path) {
                    continue;
                }
                files.push((file_path, watch_root.clone()));
            }
        }

        result.found = files.len();
        for (path, watch_root) in &files {
            match self.index_one(path, watch_root) {
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
        let (watch_root, is_ignored) = {
            let Some(entry) = self.find_entry(path) else {
                return;
            };
            let ig = build_ignore(&entry.path, path);
            let ignored = ig.matched_path_or_any_parents(path, false).is_ignore();
            (entry.path.clone(), ignored)
        };

        // Only care about .typ files in both branches.
        if path.extension().is_none_or(|ext| ext != "typ") {
            return;
        }

        if path.exists() {
            if !path.is_file() || is_ignored {
                return;
            }
            match self.index_one(path, &watch_root) {
                Ok(true) => info!("indexed {}", path.display()),
                Ok(false) => debug!("unchanged {}", path.display()),
                Err(err) => warn!("error indexing {}: {err}", path.display()),
            }
        } else {
            // File was deleted — remove from store using absolute path.
            match self.store.remove_task_file(path) {
                Ok(()) => info!("removed {}", path.display()),
                Err(err) => warn!("error removing {}: {err}", path.display()),
            }
            if let Err(err) = self.store.remove_file_error(path) {
                warn!("error removing file error for {}: {err}", path.display());
            }
        }
    }

    /// Add a new watch entry if not already watched (dedup by canonical path).
    ///
    /// Returns `Some((path, recursive))` for the caller to register with
    /// notify and scan, or `None` if already watched.
    fn add_entry(&mut self, entry: &WatchEntry) -> Result<Option<(PathBuf, bool)>, WatchError> {
        let resolved = resolve_entry(entry)?;
        if self.entries.iter().any(|e| e.path == resolved.path) {
            return Ok(None);
        }
        let result = (resolved.path.clone(), resolved.recursive);
        self.entries.push(resolved);
        Ok(Some(result))
    }

    /// Remove entries whose canonical path is not in `keep`.
    /// Returns the paths that were removed (for `unwatch()` calls).
    fn remove_entries_not_in(&mut self, keep: &HashSet<PathBuf>) -> Vec<PathBuf> {
        let mut removed = Vec::new();
        self.entries.retain(|e| {
            if keep.contains(&e.path) {
                true
            } else {
                removed.push(e.path.clone());
                false
            }
        });
        removed
    }

    /// Reload config from one or more files and update watched paths.
    ///
    /// Parse errors are non-fatal: a warning is logged and the current
    /// config is kept.
    fn reload_config(&mut self, config_paths: &[PathBuf], notify_watcher: &mut RecommendedWatcher) {
        let new_config = match config::load_and_merge(config_paths) {
            Ok(cfg) => cfg,
            Err(err) => {
                warn!("failed to reload config: {err}");
                return;
            }
        };

        // Resolve all new entries to canonical paths for diffing.
        let mut new_canonical: HashSet<PathBuf> = HashSet::new();
        for entry in &new_config.watch {
            let path = config::expand_tilde(&entry.path);
            if let Ok(canon) = std::fs::canonicalize(&path) {
                new_canonical.insert(canon);
            }
        }

        // Remove entries no longer in config.
        let removed = self.remove_entries_not_in(&new_canonical);
        for path in &removed {
            if let Err(err) = notify_watcher.unwatch(path) {
                warn!("failed to unwatch {}: {err}", path.display());
            } else {
                info!("unwatched {}", path.display());
            }
        }

        // Add new entries.
        let mut new_roots = Vec::new();
        for entry in &new_config.watch {
            match self.add_entry(entry) {
                Ok(Some((path, recursive))) => {
                    let mode = if recursive {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    };
                    if let Err(err) = notify_watcher.watch(&path, mode) {
                        warn!("failed to watch {}: {err}", path.display());
                    } else {
                        info!("watching {}", path.display());
                        new_roots.push(path);
                    }
                }
                Ok(None) => {} // already watched
                Err(err) => warn!("failed to resolve watch entry: {err}"),
            }
        }

        // Scan newly added directories.
        if !new_roots.is_empty() {
            let scan = self.scan_entries(&new_roots);
            info!(
                "config reload scan: {} found, {} indexed, {} skipped, {} errors",
                scan.found, scan.indexed, scan.skipped, scan.errors,
            );
        }

        if !removed.is_empty() || !new_roots.is_empty() {
            info!(
                "config reloaded: {} entries ({} added, {} removed)",
                self.entries.len(),
                new_roots.len(),
                removed.len(),
            );
        }
    }

    /// Start the file watcher event loop (blocks forever).
    ///
    /// Watches the given config files for changes and dynamically updates
    /// watched paths on config reload.
    ///
    /// Filters out `Access` events (e.g. file opens) to prevent infinite
    /// re-indexing loops caused by our own reads triggering inotify.
    /// Applies a simple per-path debounce window to coalesce rapid edits.
    ///
    /// # Errors
    ///
    /// Returns [`WatchError::Notify`] if the watcher cannot be created
    /// or a watched path cannot be registered.
    pub fn run(mut self, config_paths: &[PathBuf]) -> Result<(), WatchError> {
        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher: RecommendedWatcher = NotifyWatcher::new(tx, notify::Config::default())?;

        for entry in &self.entries {
            let mode = if entry.recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            watcher.watch(&entry.path, mode)?;
            info!("watching {}", entry.path.display());
        }

        // Watch each config file's parent directory so atomic-save editors
        // (write-tmp + rename) are detected correctly. Dedup parents to
        // avoid double-watching.
        let config_set: HashSet<&PathBuf> = config_paths.iter().collect();
        let mut watched_parents: HashSet<PathBuf> = HashSet::new();
        for cfg_path in config_paths {
            if let Some(parent) = cfg_path.parent() {
                if watched_parents.insert(parent.to_path_buf()) {
                    watcher.watch(parent, RecursiveMode::NonRecursive)?;
                }
                info!("watching config {}", cfg_path.display());
            }
        }

        let debounce = Duration::from_millis(300);
        let debounce_expiry = Duration::from_secs(60);
        let mut last_seen: HashMap<PathBuf, Instant> = HashMap::new();
        let mut event_count: u64 = 0;

        for result in rx {
            match result {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
                        continue;
                    }
                    let now = Instant::now();

                    // Periodically prune stale debounce entries to bound memory.
                    event_count += 1;
                    if event_count.is_multiple_of(1000) {
                        last_seen.retain(|_, &mut ts| now.duration_since(ts) < debounce_expiry);
                    }
                    for path in event.paths {
                        if is_inside_dotgit(&path) {
                            continue;
                        }
                        if let Some(&last) = last_seen.get(&path)
                            && now.duration_since(last) < debounce
                        {
                            trace!("debounced {}", path.display());
                            continue;
                        }
                        last_seen.insert(path.clone(), now);

                        // Check if this is a config file change.
                        if config_set.contains(&path) {
                            info!("config file changed, reloading");
                            self.reload_config(config_paths, &mut watcher);
                            continue;
                        }

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
    fn index_one(&mut self, path: &Path, watch_root: &Path) -> Result<bool, StoreError> {
        let world = MindTapeWorld::new(path)?;
        store::index_file_with_deps(&mut self.store, &world, path, Some(watch_root))
    }

    /// Find which resolved entry contains the given path.
    fn find_entry(&self, path: &Path) -> Option<&ResolvedEntry> {
        self.entries
            .iter()
            .find(|entry| path.starts_with(&entry.path))
    }
}

/// Resolve a single `WatchEntry` to a `ResolvedEntry` with canonical path
/// and discovered project root.
fn resolve_entry(entry: &WatchEntry) -> Result<ResolvedEntry, WatchError> {
    let path = config::expand_tilde(&entry.path);
    let path = std::fs::canonicalize(&path).map_err(|err| {
        WatchError::Other(format!("cannot resolve watch path {}: {err}", entry.path))
    })?;
    Ok(ResolvedEntry {
        path,
        recursive: entry.recursive,
    })
}

fn is_typ_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "typ") && path.is_file()
}

/// Check if a path is inside a `.git` directory.
fn is_inside_dotgit(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == ".git")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests;
