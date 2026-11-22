//! Persistent content-hash task cache
//! (`truth/tasks/2026-07/persistent-content-hash-task-cache.typ`).
//!
//! Warm `mt ls` / `mt check` re-stat and re-hash each task file, then read its
//! harvested marker straight from `<root>/.mindtape/cache/` instead of
//! re-evaluating the file through Typst. Only files whose bytes changed since
//! the last run miss the cache and pay for a fresh eval. This stacks with the
//! cold-path batch harvester ([`typst_harvest::batch_harvest`]), which the
//! loader runs over the misses alone.
//!
//! # Key
//!
//! The cache addresses an entry by the file's content hash under a
//! per-*build-key* directory. The build key folds everything that could change
//! a harvest for the same bytes: the cache format version, the `mt` version,
//! the whole config, and the prelude/`lib` contents. The config's
//! `[[typst.local-packages]]` overrides steer package resolution, so they fold
//! in too. Any build-key change routes reads to a fresh (empty) directory,
//! which busts the whole cache at once, and the loader prunes the stale
//! directories. The key folds an installed package directory by path, not
//! contents, so rewriting a version's files in place is not a cache miss. Bump
//! the version rather than edit one already published.
//!
//! # Imports
//!
//! The file's bytes are not the whole input, since a task that imports a module
//! evaluates that module too. Each entry records the content hash of every
//! project-local file the eval read
//! ([`typst_harvest::Harvest::dependencies`]). A read re-hashes them and treats
//! any mismatch, or a file gone missing, as a miss. An unreadable dependency at
//! write time leaves the entry unwritten.
//!
//! # Correctness
//!
//! A cache read never surfaces a wrong task. An unreadable, truncated,
//! wrong-format, or otherwise unparseable entry is a miss, and a miss always
//! falls back to a live eval. A harvest *error* never reaches the cache, only a
//! clean outcome (a task marker, or a confirmed absence of one). The loader then
//! re-tries a file that failed to evaluate on every run. The reconstructed
//! marker goes through the exact same `Task::from_marker` path as a cold
//! harvest, so the warm and cold results cannot drift. The cache stores the
//! Typst warnings of a clean evaluation beside the marker and replays them on
//! every hit. A warning then shows on each run, not only the first.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use typst_harvest::HVal;

use super::LoadWarning;
use crate::config::Config;
use crate::model::BodyMarker;

/// On-disk entry format. Bump when the entry shape or the marker reconstruction
/// changes. The build key folds it in, and a bump busts every entry.
const FORMAT_VERSION: u32 = 4;

/// Set this env var to a non-empty value to bypass the cache entirely (always
/// eval). For benchmarking the cold path and as an escape hatch.
const DISABLE_ENV: &str = "MINDTAPE_NO_CACHE";

/// A cache hit's payload. The file either harvested to a task marker or
/// evaluated cleanly to no `mindtape.task` marker at all.
pub(crate) struct Cached {
    /// The marker, or its confirmed absence.
    pub(crate) outcome: CachedOutcome,
    /// Every body marker the file emitted besides the task marker, in document
    /// order, the list [`crate::model::Task::markers`] takes from a hit.
    pub(crate) markers: Vec<BodyMarker>,
    /// The Typst warnings the evaluation raised, replayed on every hit.
    pub(crate) warnings: Vec<LoadWarning>,
}

/// What a cached evaluation yielded.
pub(crate) enum CachedOutcome {
    /// The reconstructed `mindtape.task` marker value. Feed it to
    /// `Task::from_marker`.
    Marker(HVal),
    /// The file evaluated without a task marker, a hit with no task.
    NoMarker,
}

/// The resolved cache directory for one project load.
pub(crate) struct TaskCache {
    /// The project root the entries' dependency paths are relative to.
    root: PathBuf,
    /// `<root>/.mindtape/cache/<build_key>`, or `None` when disabled or the
    /// directory creation failed (the cache degrades to a no-op).
    dir: Option<PathBuf>,
}

impl TaskCache {
    /// Open (and lazily create) the cache for `root` under the given `config`
    /// and package `overrides`. Best-effort: a filesystem failure disables the
    /// cache rather than failing the load.
    pub(crate) fn open(root: &Path, config: &Config, overrides: &[(String, PathBuf)]) -> Self {
        if disabled() {
            return Self {
                root: root.to_path_buf(),
                dir: None,
            };
        }
        let cache_root = root.join(".mindtape").join("cache");
        let key = build_key(config, overrides, crate::package_root().as_deref());
        prune_stale(&cache_root, &key);
        let dir = cache_root.join(&key);
        let dir = std::fs::create_dir_all(&dir).is_ok().then(|| {
            ensure_ignored(&cache_root);
            dir
        });
        Self {
            root: root.to_path_buf(),
            dir,
        }
    }

    /// The content hash addressing a file's cache entry: the SHA-1 of its bytes.
    pub(crate) fn content_hash(bytes: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        hex(&hasher.finalize())
    }

    /// The cached outcome for `content_hash`, or `None` on a miss (absent,
    /// unreadable, or unparseable, any of which falls back to a live eval).
    pub(crate) fn get(&self, content_hash: &str) -> Option<Cached> {
        let path = self.entry_path(content_hash)?;
        let text = std::fs::read_to_string(path).ok()?;
        let entry: Entry = serde_json::from_str(&text).ok()?;
        if entry.format != FORMAT_VERSION || !self.deps_unchanged(&entry.deps) {
            return None;
        }
        let outcome = match entry.marker {
            Some(value) => CachedOutcome::Marker(value),
            None => CachedOutcome::NoMarker,
        };
        Some(Cached {
            outcome,
            markers: entry.markers,
            warnings: entry.warnings,
        })
    }

    /// Whether every recorded dependency still hashes to what the entry saw.
    fn deps_unchanged(&self, deps: &[Dep]) -> bool {
        deps.iter().all(|dep| {
            std::fs::read(self.root.join(&dep.path))
                .is_ok_and(|bytes| Self::content_hash(&bytes) == dep.hash)
        })
    }

    /// Record a clean harvest outcome for `content_hash`, the payload a later
    /// [`Self::get`] returns. It holds the [`CachedOutcome`] (a task marker or
    /// a file without one), every other body marker the file emitted, and the
    /// warnings its evaluation raised. The entry also holds the project-local
    /// `deps` (relative to the root) that it imported. The write is
    /// best-effort and atomic (write-temp-then-rename). A write failure, or a
    /// dependency that the loader cannot read, leaves the entry a miss.
    pub(crate) fn put(&self, content_hash: &str, cached: &Cached, deps: &[PathBuf]) {
        let Some(path) = self.entry_path(content_hash) else {
            return;
        };
        let Some(deps) = deps
            .iter()
            .map(|rel| {
                let bytes = std::fs::read(self.root.join(rel)).ok()?;
                Some(Dep {
                    path: rel.clone(),
                    hash: Self::content_hash(&bytes),
                })
            })
            .collect::<Option<Vec<_>>>()
        else {
            return;
        };
        let entry = Entry {
            format: FORMAT_VERSION,
            marker: match &cached.outcome {
                CachedOutcome::Marker(value) => Some(value.clone()),
                CachedOutcome::NoMarker => None,
            },
            markers: cached.markers.clone(),
            warnings: cached.warnings.clone(),
            deps,
        };
        let Ok(json) = serde_json::to_string(&entry) else {
            return;
        };
        let _ = crate::fs_atomic::write_atomic(&path, json.as_bytes());
    }

    fn entry_path(&self, content_hash: &str) -> Option<PathBuf> {
        self.dir
            .as_ref()
            .map(|dir| dir.join(format!("{content_hash}.json")))
    }
}

/// One on-disk cache entry. It holds the format tag, the harvested marker as
/// JSON (`null` when the file had no task marker), and the Typst warnings the
/// evaluation raised. The other body markers the file emitted sit beside them.
#[derive(Serialize, Deserialize)]
struct Entry {
    format: u32,
    marker: Option<HVal>,
    /// Every body marker except the task marker, in document order.
    #[serde(default)]
    markers: Vec<BodyMarker>,
    #[serde(default)]
    warnings: Vec<LoadWarning>,
    /// Imported project files and their hashes at the time of the write.
    #[serde(default)]
    deps: Vec<Dep>,
}

/// One imported file an entry depends on.
#[derive(Serialize, Deserialize)]
struct Dep {
    /// Relative to the project root.
    path: PathBuf,
    /// [`TaskCache::content_hash`] of its bytes.
    hash: String,
}

/// Whether [`DISABLE_ENV`] switches the cache off.
pub(crate) fn disabled() -> bool {
    std::env::var_os(DISABLE_ENV).is_some_and(|value| !value.is_empty())
}

/// The build key, a hex digest folding the cache format, the `mt` version, the
/// whole config, `package_root` (the local-package directory), and every
/// prelude/override directory's contents. It governs the cache directory, and
/// any change busts the whole cache.
fn build_key(
    config: &Config,
    overrides: &[(String, PathBuf)],
    package_root: Option<&Path>,
) -> String {
    let mut hasher = Sha1::new();
    hasher.update(FORMAT_VERSION.to_le_bytes());
    hasher.update(b"\x00mt\x00");
    hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
    // Where an unoverridden `@local` import resolves from. The path, not its
    // contents: it holds every installed package, and only the one this project
    // imports could matter. Folding it in means moving `$MINDTAPE_DATA_HOME` or
    // `$TYPST_PACKAGE_PATH` routes reads to a fresh cache instead of replaying
    // markers harvested against a prelude that is no longer the one resolved.
    hasher.update(b"\x00pkgroot\x00");
    if let Some(dir) = package_root {
        hasher.update(dir.as_os_str().as_encoded_bytes());
    }
    hasher.update(b"\x00cfg\x00");
    // Debug is deterministic (every collection in `Config` is ordered) and
    // covers every field, so the key busts conservatively on any config change.
    hasher.update(format!("{config:?}").as_bytes());
    for (name, dir) in overrides {
        hasher.update(b"\x00pkg\x00");
        hasher.update(name.as_bytes());
        hash_dir(&mut hasher, dir);
    }
    hex(&hasher.finalize())
}

/// Fold every regular file under `dir` (sorted by relative path) into `hasher`.
/// The digest then tracks the prelude's exact contents. A missing directory
/// leaves the digest unchanged, and a lib that later appears changes it.
fn hash_dir(hasher: &mut Sha1, dir: &Path) {
    let mut files = Vec::new();
    collect_files(dir, Path::new(""), &mut files);
    files.sort();
    for (rel, path) in files {
        hasher.update(b"\x00f\x00");
        hasher.update(rel.to_string_lossy().as_bytes());
        if let Ok(bytes) = std::fs::read(&path) {
            // A fixed-width per-file digest keeps concatenation unambiguous
            // without a length prefix (and no fallible integer cast).
            hasher.update(TaskCache::content_hash(&bytes).as_bytes());
        }
    }
}

/// Collect `(relative_path, full_path)` for every regular file under `dir`,
/// recursively. `rel` accumulates the path relative to the walk's origin.
pub(crate) fn collect_files(dir: &Path, rel: &Path, out: &mut Vec<(PathBuf, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let child_rel = rel.join(entry.file_name());
        if path.is_dir() {
            collect_files(&path, &child_rel, out);
        } else {
            out.push((child_rel, path));
        }
    }
}

/// Remove sibling build-key directories under `cache_root`, keeping only
/// `current`. Best-effort: a build-key change leaves at most the fresh
/// directory behind. The prune touches only 40-hex-named directories, so a
/// stray file such as the `.gitignore` survives.
fn prune_stale(cache_root: &Path, current: &str) {
    let Ok(entries) = std::fs::read_dir(cache_root) else {
        return;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        if name != current && is_build_key(&name) && entry.path().is_dir() {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// Whether `name` looks like a build-key directory (a 40-char SHA-1 hex).
fn is_build_key(name: &str) -> bool {
    name.len() == 40 && name.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Ensure `<cache_root>/.gitignore` ignores the whole cache, so no project ever
/// tracks the scratch entries. That holds cross-project, without an `mt init`
/// edit or a root `.gitignore` line.
pub(crate) fn ensure_ignored(cache_root: &Path) {
    let ignore = cache_root.join(".gitignore");
    if !ignore.exists() {
        let _ = crate::fs_atomic::write_atomic(&ignore, b"*\n");
    }
}

/// Lowercase hex of a digest.
pub(crate) fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
