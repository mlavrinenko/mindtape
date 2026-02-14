//! File watcher for MindTape.
//!
//! Watches configured directories for `.typ` file changes, re-indexes
//! modified files, and removes deleted files from the store.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::WalkBuilder;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
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
    ignore: Gitignore,
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
    pub fn new(store: SqliteStore, entries: &[WatchEntry]) -> Result<Self, WatchError> {
        let mut resolved = Vec::with_capacity(entries.len());
        for entry in entries {
            let path = crate::config::expand_tilde(&entry.path);
            let path = std::fs::canonicalize(&path).map_err(|e| {
                WatchError::Other(format!(
                    "cannot resolve watch path {}: {e}",
                    entry.path
                ))
            })?;
            let project_root = world::find_project_root(&path);
            let ignore = build_ignore(&path);
            resolved.push(ResolvedEntry {
                path,
                project_root,
                recursive: entry.recursive,
                ignore,
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
                .git_ignore(false)
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
                Err(e) => {
                    eprintln!("error indexing {}: {e}", path.display());
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
            let ignored = entry
                .ignore
                .matched_path_or_any_parents(path, false)
                .is_ignore();
            (entry.project_root.clone(), ignored)
        };

        if path.exists() {
            if !is_typ_file(path) {
                return;
            }
            if is_ignored {
                return;
            }
            match self.index_one(path, &project_root) {
                Ok(true) => eprintln!("indexed {}", path.display()),
                Ok(false) => {}
                Err(e) => eprintln!("error indexing {}: {e}", path.display()),
            }
        } else {
            // File was deleted — remove from store.
            let rel = match path.strip_prefix(&project_root) {
                Ok(r) => r,
                Err(_) => return,
            };
            match self.store.remove_task_file(rel) {
                Ok(()) => eprintln!("removed {}", path.display()),
                Err(e) => eprintln!("error removing {}: {e}", path.display()),
            }
        }
    }

    /// Start the file watcher event loop (blocks forever).
    pub fn run(mut self) -> Result<(), WatchError> {
        let (tx, rx) = std::sync::mpsc::channel::<DebounceEventResult>();
        let mut debouncer = new_debouncer(Duration::from_millis(300), tx)?;

        for entry in &self.entries {
            let mode = if entry.recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            debouncer.watcher().watch(&entry.path, mode)?;
            eprintln!("watching {}", entry.path.display());
        }

        for batch in rx {
            match batch {
                Ok(events) => {
                    for event in events {
                        self.handle_event(&event.path);
                    }
                }
                Err(e) => {
                    eprintln!("watch error: {e}");
                }
            }
        }

        Ok(())
    }

    /// Index a single file using the existing store pipeline.
    fn index_one(&mut self, path: &Path, project_root: &Path) -> Result<bool, StoreError> {
        let world = MindTapeWorld::new(path)
            .map_err(|e| StoreError::Eval(e))?;
        store::index_file(&mut self.store, &world, path, project_root)
    }

    /// Find which resolved entry contains the given path.
    fn find_entry(&self, path: &Path) -> Option<&ResolvedEntry> {
        self.entries.iter().find(|e| path.starts_with(&e.path))
    }
}

/// Build a `Gitignore` matcher from `.mindtapeignore` in the given directory.
fn build_ignore(dir: &Path) -> Gitignore {
    let ignore_path = dir.join(".mindtapeignore");
    let mut builder = GitignoreBuilder::new(dir);
    if ignore_path.exists() {
        builder.add(&ignore_path);
    }
    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

fn is_typ_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "typ") && path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::SqliteStore;
    use std::fs;

    fn setup_watch_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        // Create project root markers.
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        fs::create_dir_all(dir.path().join("lib")).unwrap();
        fs::write(
            dir.path().join("lib/typst.toml"),
            "[package]\nname = \"mindtape\"\nversion = \"0.1.0\"\nentrypoint = \"prelude.typ\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("lib/prelude.typ"),
            "#let due(date) = metadata((\"due\", date))\n#let id(uuid) = metadata((\"id\", uuid))\n#let tag(name) = metadata((\"tag\", name))\n",
        )
        .unwrap();
        // Ignore lib/ so prelude.typ doesn't get indexed as a task file.
        fs::write(dir.path().join(".mindtapeignore"), "lib/\n").unwrap();
        let dir_path = dir.path().to_path_buf();
        (dir, dir_path)
    }

    fn make_entry(path: &Path) -> WatchEntry {
        WatchEntry {
            path: path.to_string_lossy().to_string(),
            recursive: true,
        }
    }

    // --- is_typ_file ---

    #[test]
    fn is_typ_file_true_for_typ() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("test.typ");
        fs::write(&f, "").unwrap();
        assert!(is_typ_file(&f));
    }

    #[test]
    fn is_typ_file_false_for_other_ext() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("test.rs");
        fs::write(&f, "").unwrap();
        assert!(!is_typ_file(&f));
    }

    #[test]
    fn is_typ_file_false_for_directory() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path().join("subdir.typ");
        fs::create_dir(&d).unwrap();
        assert!(!is_typ_file(&d));
    }

    #[test]
    fn is_typ_file_false_for_nonexistent() {
        assert!(!is_typ_file(Path::new("/nonexistent/file.typ")));
    }

    // --- build_ignore ---

    #[test]
    fn build_ignore_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let ig = build_ignore(dir.path());
        assert!(ig.is_empty());
    }

    #[test]
    fn build_ignore_with_patterns() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".mindtapeignore"), "*.bak\ndrafts/\n").unwrap();
        let ig = build_ignore(dir.path());
        assert!(!ig.is_empty());
        assert!(ig.matched("test.bak", false).is_ignore());
        assert!(!ig.matched("test.typ", false).is_ignore());
    }

    // --- Watcher::new ---

    #[test]
    fn watcher_new_resolves_entries() {
        let (dir, dir_path) = setup_watch_dir();
        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let watcher = Watcher::new(store, &entries).unwrap();
        assert_eq!(watcher.entries.len(), 1);
        assert!(watcher.entries[0].path.is_absolute());
        drop(dir);
    }

    #[test]
    fn watcher_new_bad_path_errors() {
        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![WatchEntry {
            path: "/nonexistent/path".to_string(),
            recursive: true,
        }];
        assert!(Watcher::new(store, &entries).is_err());
    }

    // --- initial_scan ---

    #[test]
    fn initial_scan_indexes_typ_files() {
        let (dir, dir_path) = setup_watch_dir();
        fs::write(dir_path.join("todo.typ"), "- [ ] Buy milk\n").unwrap();
        fs::write(dir_path.join("notes.txt"), "not a typ file").unwrap();

        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let mut watcher = Watcher::new(store, &entries).unwrap();
        let result = watcher.initial_scan();

        assert_eq!(result.found, 1); // only .typ files
        assert_eq!(result.indexed, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.errors, 0);
        drop(dir);
    }

    #[test]
    fn initial_scan_skips_unchanged() {
        let (dir, dir_path) = setup_watch_dir();
        fs::write(dir_path.join("todo.typ"), "- [ ] Buy milk\n").unwrap();

        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let mut watcher = Watcher::new(store, &entries).unwrap();

        let r1 = watcher.initial_scan();
        assert_eq!(r1.indexed, 1);

        let r2 = watcher.initial_scan();
        assert_eq!(r2.skipped, 1);
        assert_eq!(r2.indexed, 0);
        drop(dir);
    }

    #[test]
    fn initial_scan_respects_mindtapeignore() {
        let (dir, dir_path) = setup_watch_dir();
        // Overwrite the default .mindtapeignore to also ignore ignored.typ.
        fs::write(dir_path.join(".mindtapeignore"), "lib/\nignored.typ\n").unwrap();
        fs::write(dir_path.join("todo.typ"), "- [ ] Task\n").unwrap();
        fs::write(dir_path.join("ignored.typ"), "- [ ] Hidden\n").unwrap();

        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let mut watcher = Watcher::new(store, &entries).unwrap();
        let result = watcher.initial_scan();

        assert_eq!(result.found, 1); // ignored.typ filtered by walker
        drop(dir);
    }

    // --- handle_event ---

    #[test]
    fn handle_event_indexes_new_file() {
        let (dir, dir_path) = setup_watch_dir();
        let typ_file = dir_path.join("new.typ");
        fs::write(&typ_file, "- [ ] New task\n").unwrap();

        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let mut watcher = Watcher::new(store, &entries).unwrap();
        watcher.handle_event(&typ_file);

        let tasks = watcher
            .store
            .query_tasks(&crate::store::TaskFilter::default())
            .unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "New task");
        drop(dir);
    }

    #[test]
    fn handle_event_removes_deleted_file() {
        let (dir, dir_path) = setup_watch_dir();
        let typ_file = dir_path.join("del.typ");
        fs::write(&typ_file, "- [ ] Will be deleted\n").unwrap();

        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let mut watcher = Watcher::new(store, &entries).unwrap();

        // Index it first.
        watcher.handle_event(&typ_file);
        let tasks = watcher
            .store
            .query_tasks(&crate::store::TaskFilter::default())
            .unwrap();
        assert_eq!(tasks.len(), 1);

        // Delete the file, then handle the event.
        fs::remove_file(&typ_file).unwrap();
        watcher.handle_event(&typ_file);

        let tasks = watcher
            .store
            .query_tasks(&crate::store::TaskFilter::default())
            .unwrap();
        assert_eq!(tasks.len(), 0);
        drop(dir);
    }

    #[test]
    fn handle_event_ignores_non_typ() {
        let (dir, dir_path) = setup_watch_dir();
        let txt_file = dir_path.join("notes.txt");
        fs::write(&txt_file, "not typ").unwrap();

        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let mut watcher = Watcher::new(store, &entries).unwrap();
        watcher.handle_event(&txt_file);

        let tasks = watcher
            .store
            .query_tasks(&crate::store::TaskFilter::default())
            .unwrap();
        assert_eq!(tasks.len(), 0);
        drop(dir);
    }

    #[test]
    fn handle_event_ignores_mindtapeignored_file() {
        let (dir, dir_path) = setup_watch_dir();
        fs::write(dir_path.join(".mindtapeignore"), "secret.typ\n").unwrap();
        let typ_file = dir_path.join("secret.typ");
        fs::write(&typ_file, "- [ ] Secret task\n").unwrap();

        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let mut watcher = Watcher::new(store, &entries).unwrap();
        watcher.handle_event(&typ_file);

        let tasks = watcher
            .store
            .query_tasks(&crate::store::TaskFilter::default())
            .unwrap();
        assert_eq!(tasks.len(), 0);
        drop(dir);
    }

    #[test]
    fn handle_event_unknown_path_is_noop() {
        let (dir, dir_path) = setup_watch_dir();
        let store = SqliteStore::open_memory().unwrap();
        let entries = vec![make_entry(&dir_path)];
        let mut watcher = Watcher::new(store, &entries).unwrap();
        // Path outside any watched entry.
        watcher.handle_event(Path::new("/tmp/other/file.typ"));
        drop(dir);
    }
}
