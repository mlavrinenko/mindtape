//! `.mindtape/packs.lock`: for each pack source, the `git` and `ref` it was
//! locked from, the `commit` that `ref` named then, and the `sha256` of the
//! source's files at that commit. `mt packs update` writes it with
//! [`write_lock`], and the project commits it.

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::ConfigError;
use crate::config::pack_sources::PackSource;

/// The lock file's path under a project root.
#[must_use]
pub fn lock_path(root: &Path) -> PathBuf {
    root.join(".mindtape").join("packs.lock")
}

/// One source as the lock pins it, `[<name>]` in the lock file.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LockedSource {
    /// The repository the source was locked from.
    pub git: String,
    /// The tag, branch or commit the source was locked from.
    #[serde(rename = "ref")]
    pub reference: String,
    /// The commit `reference` named when the lock was written, in hex.
    pub commit: String,
    /// The hash of the source's files at `commit`, in lowercase hex.
    pub sha256: String,
}

/// The whole lock file, keyed by source name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackLock {
    /// Each locked source, by name.
    pub entries: BTreeMap<String, LockedSource>,
}

impl PackLock {
    /// Whether the lock pins `name` from `source`'s own `git` and `ref`.
    #[must_use]
    pub fn pins(&self, name: &str, source: &PackSource) -> bool {
        self.entries
            .get(name)
            .is_some_and(|locked| locked.git == source.git && locked.reference == source.reference)
    }
}

/// The comment a written lock starts with.
const HEADER: &str = "# Written by `mt packs update`. Commit it beside .mindtape/config.toml.\n\n";

/// Write `lock` as the lock file under `root`, all at once, so a reader sees
/// the old file or the new one and never a part of it.
///
/// # Errors
/// Returns the I/O error when the file can't be written.
pub fn write_lock(root: &Path, lock: &PackLock) -> std::io::Result<()> {
    let body = toml::to_string(&lock.entries).map_err(std::io::Error::other)?;
    crate::write_atomic(&lock_path(root), format!("{HEADER}{body}").as_bytes())
}

/// Read the lock under `root`. A missing file is an empty lock.
///
/// # Errors
/// Returns [`ConfigError::ParseIn`] naming the file when it is not a lock,
/// [`ConfigError::Invalid`] for a `commit` or `sha256` that is not hex of the
/// right length, and [`ConfigError::Io`] when it can't be read.
pub(crate) fn read_lock(root: &Path) -> Result<PackLock, ConfigError> {
    let text = match std::fs::read_to_string(lock_path(root)) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(PackLock::default()),
        Err(error) => return Err(error.into()),
    };
    let shown = lock_path(Path::new(""));
    let entries: BTreeMap<String, LockedSource> =
        toml::from_str(&text).map_err(|error| ConfigError::ParseIn {
            origin: shown.display().to_string(),
            error,
        })?;
    for (name, locked) in &entries {
        check_hex(&shown, name, "commit", &locked.commit, &[40, 64])?;
        check_hex(&shown, name, "sha256", &locked.sha256, &[64])?;
    }
    Ok(PackLock { entries })
}

/// Refuse `value` unless it is lowercase hex of one of `lengths`, since a
/// `sha256` names a directory and a `commit` goes to `git`.
fn check_hex(
    shown: &Path,
    name: &str,
    key: &str,
    value: &str,
    lengths: &[usize],
) -> Result<(), ConfigError> {
    let hex = value
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'));
    if hex && lengths.contains(&value.len()) {
        return Ok(());
    }
    Err(ConfigError::invalid(format!(
        "{}: {name}.{key}: not lowercase hex of the right length: {}",
        shown.display(),
        crate::config::quote(value)
    )))
}
