//! `at.toml`: the app-scoped file that names projects and sets of projects,
//! which the CLI's leading `@name` selects (`mt @home ls`).
//!
//! It picks *which* projects `mt` loads, so it is parsed on its own from the
//! app config dir and never merged into a project [`crate::config::Config`].
//! Each `[at.<name>]` entry holds `include`, `exclude` and `forks`. An entry
//! whose `include` is one literal path is a project; any other entry is a set.
//!
//! - `mod.rs` — the entry types, parsing and errors.
//! - `resolve.rs` — `@x` to a project or a set: `@name` expansion, `exclude`,
//!   `forks`, the directory-name fallback and member naming.
//! - `scan.rs` — the injected filesystem ([`AtFs`], [`DiskFs`]), `~` expansion,
//!   one-level glob expansion and the project and fork tests.
//! - `tests.rs` — the unit tests, over an in-memory filesystem.
//! - `scan_tests.rs` — the [`DiskFs`] tests, over a temporary tree.

mod resolve;
mod scan;

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer};
use thiserror::Error;

pub use scan::{AtFs, DiskFs};

/// A parsed `at.toml`: its entries by name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AtConfig {
    entries: BTreeMap<String, AtEntry>,
}

/// One `[at.<name>]` entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtEntry {
    /// Literal paths, one-level `*` globs, or `@name`s.
    #[serde(default)]
    pub include: Vec<String>,
    /// The same forms as `include`, whose matches leave the set.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Whether a glob's linked git worktrees join the set.
    #[serde(default)]
    pub forks: Forks,
}

/// What `forks` says about the linked git worktrees a glob matches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Forks {
    /// `false`, the default: forks stay out.
    #[default]
    Skip,
    /// `true`: forks join beside their main checkouts.
    Keep,
    /// `"only"`: only forks join.
    Only,
}

impl<'de> Deserialize<'de> for Forks {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Bool(bool),
            Word(String),
        }
        match Raw::deserialize(deserializer)? {
            Raw::Bool(false) => Ok(Self::Skip),
            Raw::Bool(true) => Ok(Self::Keep),
            Raw::Word(word) if word == "only" => Ok(Self::Only),
            Raw::Word(word) => Err(serde::de::Error::custom(format!(
                "forks must be true, false or \"only\", not {word:?}"
            ))),
        }
    }
}

/// What `@x` resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// One project, from a one-path entry or a directory name.
    Project(PathBuf),
    /// A set, in include order, even when it holds one member.
    Set(Vec<Member>),
}

/// One project of a set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// Its directory name, or its shortest unique path suffix on a clash.
    pub name: String,
    /// Its `~`-expanded path.
    pub path: PathBuf,
    /// Whether it is a linked git worktree.
    pub fork: bool,
}

/// The raw file shape.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    at: BTreeMap<String, AtEntry>,
}

/// An `at.toml` parse or resolution failure.
#[derive(Debug, Error)]
pub enum AtError {
    /// The file was not valid TOML or had an unknown key.
    #[error("{0}")]
    Parse(#[from] toml::de::Error),
    /// An entry is malformed.
    #[error("[at.{name}]: {message}")]
    Entry {
        /// The entry's name.
        name: String,
        /// What is wrong with it.
        message: &'static str,
    },
    /// An entry name or `@x` token is not one safe path segment.
    #[error("invalid name {name:?}: {reason}")]
    InvalidName {
        /// The offending name, without its `@`.
        name: String,
        /// Why it was refused.
        reason: &'static str,
    },
    /// An `@name` in an `include` or `exclude` names no entry and no directory.
    #[error("unknown name @{name}")]
    Unknown {
        /// The name, without its `@`.
        name: String,
    },
    /// Several directories answer to one name.
    #[error("@{name} matches {} directories", paths.len())]
    Clash {
        /// The name, without its `@`.
        name: String,
        /// Every matching directory, in config order.
        paths: Vec<PathBuf>,
    },
    /// `@name` includes reach back to an entry being expanded.
    #[error("@name cycle: {}", chain.join(" -> "))]
    Cycle {
        /// The entry names from the first to the repeated one.
        chain: Vec<String>,
    },
    /// An `include` or `exclude` glob does not compile.
    #[error("bad glob {pattern:?}: {source}")]
    Glob {
        /// The glob as written.
        pattern: String,
        /// The compiler's complaint.
        source: globset::Error,
    },
}

impl AtConfig {
    /// Parse an `at.toml` document.
    ///
    /// # Errors
    /// [`AtError::Parse`] on malformed TOML or an unknown key,
    /// [`AtError::InvalidName`] on an entry name that is not one path segment,
    /// or [`AtError::Entry`] on an entry with no `include`.
    pub fn parse(text: &str) -> Result<Self, AtError> {
        let raw: RawFile = toml::from_str(text)?;
        for (name, entry) in &raw.at {
            validate_name(name)?;
            if entry.include.is_empty() {
                return Err(AtError::Entry {
                    name: name.clone(),
                    message: "needs an `include`",
                });
            }
        }
        Ok(Self { entries: raw.at })
    }

    /// The entry names, sorted.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// The entry called `name`.
    #[must_use]
    pub fn entry(&self, name: &str) -> Option<&AtEntry> {
        self.entries.get(name)
    }
}

/// Refuse a name that is not one safe path segment.
fn validate_name(name: &str) -> Result<(), AtError> {
    let invalid = |reason| {
        Err(AtError::InvalidName {
            name: name.to_owned(),
            reason,
        })
    };
    if name.is_empty() {
        return invalid("empty name");
    }
    if name == "." || name == ".." {
        return invalid("`.` and `..` are not names");
    }
    if name.contains(['/', '\\']) {
        return invalid("must be one path segment, with no `/` or `\\`");
    }
    Ok(())
}

#[cfg(test)]
mod tests;
