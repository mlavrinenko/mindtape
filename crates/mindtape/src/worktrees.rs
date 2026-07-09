//! App-scoped worktree registry: named project bookmarks resolved by the CLI's
//! leading `@name` sugar (`mt @sn ls`).
//!
//! This selects *which* project `mt` loads, so it lives outside the per-project
//! config layer chain: it is parsed standalone from an app-config file, never
//! merged into a project [`crate::config::Config`]. The module is pure —
//! filesystem access is injected (`is_project` predicate, `home` for `~`
//! expansion) — so it stays consumer-blind and unit-testable.
//!
//! Entries are tried top-to-bottom, first match wins:
//! - An [`WorktreeEntry::Alias`] (`name` + `path`) matches on the name alone; a
//!   broken `path` surfaces a loud downstream error rather than falling through.
//! - A [`WorktreeEntry::Base`] (`base`) matches `@<token>` when `<base>/<token>`
//!   is a project (`is_project` holds), so several bases can be listed and the
//!   first that resolves wins.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// A parsed worktree registry: an ordered list of entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorktreeRegistry {
    entries: Vec<WorktreeEntry>,
}

/// One registry entry: an explicit alias or a base directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktreeEntry {
    /// `@name` → `path`, matched on the name alone.
    Alias {
        /// The token that selects this entry.
        name: String,
        /// Target worktree path (may start with `~`).
        path: PathBuf,
    },
    /// `@<token>` → `base/<token>` when that candidate is a project.
    Base {
        /// Directory the token is joined under (may start with `~`).
        base: PathBuf,
    },
}

/// The raw file shape before classification/validation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    worktrees: Vec<RawEntry>,
}

/// One raw entry: the optional-field union of both kinds, classified by
/// [`classify`].
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    name: Option<String>,
    path: Option<PathBuf>,
    base: Option<PathBuf>,
}

/// A worktrees-config parse or resolution failure.
#[derive(Debug, Error)]
pub enum WorktreeError {
    /// The file was not valid TOML or had an unknown field.
    #[error("{0}")]
    Parse(#[from] toml::de::Error),
    /// An entry mixed kinds or omitted a required field.
    #[error("worktree entry #{index}: {message}")]
    Entry {
        /// 1-based position in the `[[worktrees]]` array.
        index: usize,
        /// What was wrong with the entry.
        message: String,
    },
    /// Two alias entries declared the same name.
    #[error("duplicate worktree alias {name:?}")]
    DuplicateAlias {
        /// The clashing alias name.
        name: String,
    },
    /// The `@token` was not a single, safe path segment.
    #[error("invalid worktree name {token:?}: {reason}")]
    InvalidToken {
        /// The offending token (without the leading `@`).
        token: String,
        /// Why it was rejected.
        reason: String,
    },
}

impl WorktreeRegistry {
    /// Parse a `worktrees.toml` document into a validated registry.
    ///
    /// # Errors
    /// [`WorktreeError::Parse`] on malformed TOML, [`WorktreeError::Entry`] on a
    /// mixed or incomplete entry, or [`WorktreeError::DuplicateAlias`] on a
    /// repeated alias name.
    pub fn parse(text: &str) -> Result<Self, WorktreeError> {
        let raw: RawFile = toml::from_str(text)?;
        let mut entries = Vec::with_capacity(raw.worktrees.len());
        let mut seen = HashSet::new();
        for (i, entry) in raw.worktrees.into_iter().enumerate() {
            let entry = classify(i + 1, entry)?;
            if let WorktreeEntry::Alias { name, .. } = &entry
                && !seen.insert(name.clone())
            {
                return Err(WorktreeError::DuplicateAlias { name: name.clone() });
            }
            entries.push(entry);
        }
        Ok(Self { entries })
    }

    /// The entries in declaration order.
    #[must_use]
    pub fn entries(&self) -> &[WorktreeEntry] {
        &self.entries
    }

    /// Whether no entries are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Resolve `token` (the text after a leading `@`) to a worktree path.
    ///
    /// Entries are tried in order. An alias matches on its name and returns its
    /// `~`-expanded path unconditionally. A base returns `base/token` only when
    /// `is_project` holds for that candidate, so a base with no matching project
    /// is skipped. `home` expands a leading `~`. Returns `Ok(None)` when nothing
    /// matched.
    ///
    /// # Errors
    /// [`WorktreeError::InvalidToken`] when `token` is not a single, safe path
    /// segment.
    pub fn resolve(
        &self,
        token: &str,
        home: Option<&Path>,
        is_project: impl Fn(&Path) -> bool,
    ) -> Result<Option<PathBuf>, WorktreeError> {
        validate_token(token)?;
        for entry in &self.entries {
            match entry {
                WorktreeEntry::Alias { name, path } if name == token => {
                    return Ok(Some(expand_tilde(path, home)));
                }
                WorktreeEntry::Base { base } => {
                    let candidate = expand_tilde(base, home).join(token);
                    if is_project(&candidate) {
                        return Ok(Some(candidate));
                    }
                }
                WorktreeEntry::Alias { .. } => {}
            }
        }
        Ok(None)
    }

    /// The declared alias names, in order — for suggesting valid tokens when a
    /// resolution misses.
    #[must_use]
    pub fn alias_names(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter_map(|entry| match entry {
                WorktreeEntry::Alias { name, .. } => Some(name.as_str()),
                WorktreeEntry::Base { .. } => None,
            })
            .collect()
    }
}

/// Classify a raw entry into an alias or a base, rejecting mixed or incomplete
/// shapes. `index` is the 1-based array position, for error messages.
fn classify(index: usize, raw: RawEntry) -> Result<WorktreeEntry, WorktreeError> {
    let entry = |message: &str| WorktreeError::Entry {
        index,
        message: message.to_owned(),
    };
    match (raw.name, raw.path, raw.base) {
        (Some(_), _, Some(_)) | (_, Some(_), Some(_)) => Err(entry(
            "mixes `base` with `name`/`path`; use one or the other",
        )),
        (Some(name), Some(path), None) if !name.trim().is_empty() => {
            Ok(WorktreeEntry::Alias { name, path })
        }
        (Some(_), Some(_), None) => Err(entry("`name` is empty")),
        (Some(_), None, None) => Err(entry("alias needs a `path`")),
        (None, Some(_), None) => Err(entry("alias needs a `name`")),
        (None, None, Some(base)) => Ok(WorktreeEntry::Base { base }),
        (None, None, None) => Err(entry("empty entry; needs `name` + `path` or `base`")),
    }
}

/// Expand a leading `~` in `path` against `home`; other paths pass through.
fn expand_tilde(path: &Path, home: Option<&Path>) -> PathBuf {
    match (home, path.strip_prefix("~")) {
        (Some(home), Ok(rest)) => home.join(rest),
        _ => path.to_path_buf(),
    }
}

/// Reject a token that is not a single, safe path segment, so a base join cannot
/// escape its directory.
fn validate_token(token: &str) -> Result<(), WorktreeError> {
    let invalid = |reason: &str| {
        Err(WorktreeError::InvalidToken {
            token: token.to_owned(),
            reason: reason.to_owned(),
        })
    };
    if token.is_empty() {
        return invalid("empty name");
    }
    if token == "." || token == ".." {
        return invalid("`.` and `..` are not worktree names");
    }
    if token.contains('/') || token.contains('\\') {
        return invalid("must be a single path segment (no `/` or `\\`)");
    }
    Ok(())
}

#[cfg(test)]
#[path = "worktrees_tests.rs"]
mod tests;
