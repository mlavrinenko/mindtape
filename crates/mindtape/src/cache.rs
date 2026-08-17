//! Persistent content-hash task cache
//! (`truth/tasks/persistent-content-hash-task-cache.typ`).
//!
//! Warm `mt ls` / `mt check` over a mostly-static backlog re-stat and re-hash
//! each task file, then read its harvested marker straight from
//! `<root>/.mindtape/cache/` instead of re-evaluating the file through Typst.
//! Only files whose bytes changed since the last run miss the cache and pay for
//! a fresh eval; this stacks with the cold-path batch harvester
//! ([`typst_harvest::batch_harvest`]), which the loader now runs over the
//! misses alone.
//!
//! # Key
//!
//! An entry is addressed by the file's content hash under a per-*build-key*
//! directory. The build key folds everything that could change a harvest for
//! the same bytes: the cache format version, the `mt` version, the whole config
//! (its `[[typst.local-packages]]` overrides steer package resolution), and the
//! prelude/`lib` contents. Any build-key change routes reads to a fresh (empty)
//! directory, so the whole cache busts at once and the stale directories are
//! pruned.
//!
//! # Correctness
//!
//! A cache read never surfaces a wrong task: an unreadable, truncated,
//! wrong-format, or otherwise unparseable entry is a miss, and a miss always
//! falls back to a live eval. Harvest *errors* are never cached — only a clean
//! outcome (a task marker, or a confirmed absence of one) is stored — so a file
//! that failed to evaluate is re-tried every run. The reconstructed marker is
//! projected through the exact same `Task::from_marker` path as a cold harvest,
//! so the warm and cold results cannot drift.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha1::{Digest, Sha1};
use typst_harvest::HVal;

use crate::canonical::{hval_to_json, json_to_hval};
use crate::config::Config;

/// On-disk entry format. Bump when the entry shape or the marker reconstruction
/// changes; folded into the build key, so a bump busts every entry.
const FORMAT_VERSION: u32 = 1;

/// Set this env var to a non-empty value to bypass the cache entirely (always
/// eval). For benchmarking the cold path and as an escape hatch.
const DISABLE_ENV: &str = "MINDTAPE_NO_CACHE";

/// A cache hit's payload: the file harvested to a task marker, or evaluated
/// cleanly to no `mindtape.task` marker at all.
pub(crate) enum Cached {
    /// The reconstructed `mindtape.task` marker value; feed to
    /// `Task::from_marker`.
    Marker(HVal),
    /// The file evaluated with no task marker — a hit that yields no task.
    NoMarker,
}

/// The resolved cache directory for one project load.
pub(crate) struct TaskCache {
    /// `<root>/.mindtape/cache/<build_key>`, or `None` when disabled or the
    /// directory could not be created (the cache degrades to a no-op).
    dir: Option<PathBuf>,
}

impl TaskCache {
    /// Open (and lazily create) the cache for `root` under the given `config`
    /// and package `overrides`. Best-effort: a filesystem failure disables the
    /// cache rather than failing the load.
    pub(crate) fn open(root: &Path, config: &Config, overrides: &[(String, PathBuf)]) -> Self {
        if disabled() {
            return Self { dir: None };
        }
        let cache_root = root.join(".mindtape").join("cache");
        let key = build_key(config, overrides);
        prune_stale(&cache_root, &key);
        let dir = cache_root.join(&key);
        let dir = std::fs::create_dir_all(&dir).is_ok().then(|| {
            ensure_ignored(&cache_root);
            dir
        });
        Self { dir }
    }

    /// The content hash addressing a file's cache entry: the SHA-1 of its bytes.
    pub(crate) fn content_hash(bytes: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        hex(&hasher.finalize())
    }

    /// The cached outcome for `content_hash`, or `None` on a miss (absent,
    /// unreadable, or unparseable — any of which falls back to a live eval).
    pub(crate) fn get(&self, content_hash: &str) -> Option<Cached> {
        let path = self.entry_path(content_hash)?;
        let text = std::fs::read_to_string(path).ok()?;
        let entry: Entry = serde_json::from_str(&text).ok()?;
        if entry.format != FORMAT_VERSION {
            return None;
        }
        Some(match entry.marker {
            Some(value) => Cached::Marker(json_to_hval(&value)),
            None => Cached::NoMarker,
        })
    }

    /// Record a clean harvest outcome for `content_hash`: `Some(marker)` for a
    /// task, `None` for a file that produced no marker. Best-effort and atomic
    /// (write-temp-then-rename); a write failure just leaves the entry a miss.
    pub(crate) fn put(&self, content_hash: &str, marker: Option<&HVal>) {
        let Some(path) = self.entry_path(content_hash) else {
            return;
        };
        let entry = Entry {
            format: FORMAT_VERSION,
            marker: marker.map(hval_to_json),
        };
        let Ok(json) = serde_json::to_string(&entry) else {
            return;
        };
        write_atomic(&path, json.as_bytes());
    }

    fn entry_path(&self, content_hash: &str) -> Option<PathBuf> {
        self.dir
            .as_ref()
            .map(|dir| dir.join(format!("{content_hash}.json")))
    }
}

/// One on-disk cache entry: the format tag plus the harvested marker as JSON
/// (`null` when the file yielded no task marker).
#[derive(Serialize, Deserialize)]
struct Entry {
    format: u32,
    marker: Option<Value>,
}

/// Whether the cache is switched off via [`DISABLE_ENV`].
fn disabled() -> bool {
    std::env::var_os(DISABLE_ENV).is_some_and(|value| !value.is_empty())
}

/// The build key: a hex digest folding the cache format, the `mt` version, the
/// whole config, the local-package directory, and every prelude/override
/// directory's contents. Governs the cache directory, so any change busts the
/// whole cache.
fn build_key(config: &Config, overrides: &[(String, PathBuf)]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(FORMAT_VERSION.to_le_bytes());
    hasher.update(b"\x00mt\x00");
    hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
    // Where an unoverridden `@local` import resolves from. The path, not its
    // contents: it holds every installed package, and only the one this project
    // imports could matter. Folding it in means moving `$MINDTAPE_DATA_HOME`
    // routes reads to a fresh cache instead of replaying markers harvested
    // against a prelude that is no longer the one being resolved.
    hasher.update(b"\x00pkgroot\x00");
    if let Some(dir) = crate::data_home() {
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

/// Fold every regular file under `dir` (sorted by relative path) into `hasher`,
/// so the digest tracks the prelude's exact contents. A missing directory
/// contributes nothing; a lib that later appears changes the digest.
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
fn collect_files(dir: &Path, rel: &Path, out: &mut Vec<(PathBuf, PathBuf)>) {
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
/// directory behind. Only 40-hex-named directories are touched, so a stray file
/// (e.g. the `.gitignore`) is never removed.
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
/// tracks the scratch entries — cross-project, without an `mt init` edit or a
/// root `.gitignore` line.
fn ensure_ignored(cache_root: &Path) {
    let ignore = cache_root.join(".gitignore");
    if !ignore.exists() {
        let _ = std::fs::write(ignore, "*\n");
    }
}

/// Write `bytes` to `path` atomically: a uniquely-named temp file in the same
/// directory, then a rename. Concurrent writers (the parallel harvest) never
/// see a half-written entry. Best-effort; any failure is swallowed.
fn write_atomic(path: &Path, bytes: &[u8]) {
    let Some(dir) = path.parent() else {
        return;
    };
    let tmp = dir.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    if std::fs::write(&tmp, bytes).is_ok() && std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Lowercase hex of a digest.
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
