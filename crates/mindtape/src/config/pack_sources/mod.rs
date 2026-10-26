//! `[pack-sources]`: the git repositories a project takes packs from, and
//! `.mindtape/packs.lock`, which pins each one to a commit and a hash.
//!
//! Only the project's own `.mindtape/config.toml` declares a source, so a
//! clone resolves the same packs on every machine. [`for_layers`] reads the
//! sources on every config load, refuses them in any other layer, and refuses
//! a source the lock doesn't pin from its declared `git` and `ref`.
//! [`read_raw`] reads the same two files without loading the config, for
//! `mt packs`, which repairs the very state a load refuses. A fetched source
//! sits in [`cache_root`] under its `sha256`.
//!
//! # Files
//!
//! - `mod.rs`: `[pack-sources]`, [`read_raw`] and the load-time checks.
//! - `fetch.rs`, `fetch_tests.rs`: [`fetch`], one locked source into the
//!   cache, and [`locate`], the repository a declared `git` names.
//! - `git.rs`, `git_tests.rs`: git run apart from the caller's git state,
//!   and the blobs streamed out of it.
//! - `hash.rs`, `hash_tests.rs`: [`hash`], the framed tree hash a lock pins.
//! - `listing.rs`, `listing_tests.rs`: the blobs of a commit's tree, and
//!   the paths no platform may write differently.
//! - `lock.rs`: `.mindtape/packs.lock` and its entries.
//! - `scratch.rs`, `scratch_tests.rs`: the directory a fetch builds in, its
//!   sweep, and the rename into the cache.
//! - `tests.rs`: the unit tests, through a full config load.

mod fetch;
mod git;
mod hash;
mod listing;
mod lock;
mod scratch;

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub use crate::config::pack_sources::fetch::{FetchError, fetch, locate};
pub use crate::config::pack_sources::hash::hash;
pub use crate::config::pack_sources::lock::{LockedSource, PackLock, lock_path};

use crate::config::layers::{Layer, LayerSource};
use crate::config::pack_registry::EMBEDDED_SOURCE;
use crate::config::{ConfigError, config_path, quote};

/// The config key that declares the sources.
const KEY: &str = "pack-sources";

/// One `[pack-sources.<name>]` entry.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackSource {
    /// The repository, as any URL or path `git fetch` takes.
    pub git: String,
    /// The tag, branch or commit to lock.
    #[serde(rename = "ref")]
    pub reference: String,
}

/// The project's declared sources and its lock, read together.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackSources {
    /// Each `[pack-sources.<name>]`, by name.
    pub sources: BTreeMap<String, PackSource>,
    /// `.mindtape/packs.lock`, empty when the file is missing.
    pub lock: PackLock,
}

impl PackSources {
    /// The first source by name that the lock lacks, or pins from another
    /// `git` or `ref` than the one declared.
    #[must_use]
    pub fn stale(&self) -> Option<&str> {
        self.sources
            .iter()
            .find(|(name, source)| !self.lock.pins(name, source))
            .map(|(name, _)| name.as_str())
    }

    /// Refuse a [`PackSources::stale`] source.
    ///
    /// # Errors
    /// Returns [`ConfigError::Invalid`] naming the first stale source, with a
    /// hint to run `mt packs update`.
    pub fn check_fresh(&self) -> Result<(), ConfigError> {
        match self.stale() {
            Some(name) => Err(ConfigError::invalid(format!(
                "pack source lock is stale: {}",
                quote(name)
            ))
            .with_hint("run `mt packs update` to lock the sources as declared")),
            None => Ok(()),
        }
    }

    /// Whether the project declares a source named `name`.
    pub(crate) fn declares(&self, name: &str) -> bool {
        self.sources.contains_key(name)
    }
}

/// The directory each fetched source's tree sits in, as `<sha256>/`:
/// `mindtape/packs/` under [`crate::cache_home`]. `None` when no cache
/// directory applies, so no source is installed.
#[must_use]
pub fn cache_root() -> Option<PathBuf> {
    crate::cache_home().map(|home| home.join("mindtape").join("packs"))
}

/// Whether `name` takes only letters, digits, `-` and `_`, as a source name
/// and the pack part of a `<source>:<pack>` entry must. Such a name can't
/// leave the directory it is looked up in.
pub(crate) fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

/// The project's sources and lock under `root`, read without resolving
/// `extends` or checking the lock against the sources. A missing config or
/// lock reads as empty.
///
/// # Errors
/// Returns [`ConfigError`] when the config or the lock can't be read, isn't
/// valid TOML, or declares a malformed source or lock entry.
pub fn read_raw(root: &Path) -> Result<PackSources, ConfigError> {
    let text = match std::fs::read_to_string(config_path(root)) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    Ok(PackSources {
        sources: parse_sources(&text)?,
        lock: lock::read_lock(root)?,
    })
}

/// The sources of a config load: the project layer's `[pack-sources]` and the
/// lock under `root`, checked against each other.
///
/// # Errors
/// Returns [`ConfigError`] when a layer other than the project's declares a
/// source, a source is malformed, the lock can't be read, or the lock is
/// stale.
pub(crate) fn for_layers(layers: &[Layer], root: &Path) -> Result<PackSources, ConfigError> {
    let mut sources = BTreeMap::new();
    for layer in layers {
        match &layer.source {
            LayerSource::Project(_) => sources = parse_sources(&layer.text)?,
            LayerSource::Xdg(path) | LayerSource::Dir(path) => {
                refuse_outside_project(layer, path, root)?;
            }
            LayerSource::Embedded => {}
        }
    }
    let found = PackSources {
        sources,
        lock: lock::read_lock(root)?,
    };
    found.check_fresh()?;
    Ok(found)
}

/// A config text's `[pack-sources]`, every other key ignored.
#[derive(Deserialize)]
struct SourcesOnly {
    #[serde(default, rename = "pack-sources")]
    sources: BTreeMap<String, PackSource>,
}

/// Parse and check the `[pack-sources]` of one config text.
fn parse_sources(text: &str) -> Result<BTreeMap<String, PackSource>, ConfigError> {
    let SourcesOnly { sources } = toml::from_str(text)?;
    for name in sources.keys() {
        check_name(name)?;
    }
    Ok(sources)
}

/// Refuse `mindtape`, which names the packs built into `mt`, and a name an
/// `extends` entry couldn't spell as `<name>:<pack>`.
fn check_name(name: &str) -> Result<(), ConfigError> {
    if name == EMBEDDED_SOURCE {
        return Err(ConfigError::invalid(format!(
            "{KEY}: the name is taken by the packs built into mt: {}",
            quote(name)
        )));
    }
    if is_plain_name(name) {
        return Ok(());
    }
    Err(ConfigError::invalid(format!(
        "{KEY}: a name takes letters, digits, '-' and '_': {}",
        quote(name)
    )))
}

/// Refuse `[pack-sources]` in `layer`, which isn't the project's. A layer
/// whose TOML doesn't parse passes here, and the merge reports it.
fn refuse_outside_project(layer: &Layer, path: &Path, root: &Path) -> Result<(), ConfigError> {
    let declares =
        toml::from_str::<toml::Table>(&layer.text).is_ok_and(|table| table.contains_key(KEY));
    if !declares {
        return Ok(());
    }
    let message = format!(
        "{KEY}: declared outside the project config: {}",
        quote(&path.display().to_string())
    );
    let place = crate::config::locate::Sources::of_layers(std::slice::from_ref(layer), root)
        .place_of(&message);
    Err(ConfigError::Invalid {
        message,
        place,
        unknown: None,
        hint: Some("declare pack sources in the project's .mindtape/config.toml"),
    })
}

#[cfg(test)]
mod tests;
