//! The packs compiled into the binary and the types that say where a rule
//! came from: [`EMBEDDED_PACKS`] under the one source known without a
//! declaration, [`EMBEDDED_SOURCE`], and the origin types `mt config show
//! --origin` prints. [`crate::config::pack_resolve`] looks a `<source>:<pack>`
//! entry up here; [`crate::config::packs`] folds what it returns.

use std::path::{Path, PathBuf};

use crate::config::layers::LayerSource;

/// The source of the packs compiled into the binary: an `extends` entry
/// `mindtape:<pack>` names one. It is the one source known without a
/// declaration.
pub(crate) const EMBEDDED_SOURCE: &str = "mindtape";

/// The file a directory pack's table is read from.
pub(crate) const PACK_FILE: &str = "pack.toml";

/// Packs compiled into the binary, `(pack, TOML text)`, each under
/// [`EMBEDDED_SOURCE`]. A new pack is a `crates/mindtape/src/packs/<pack>.toml`
/// file plus a one-line entry here.
pub(crate) const EMBEDDED_PACKS: &[(&str, &str)] =
    &[("tasks", include_str!("../packs/tasks.toml"))];

/// Where one resolved pack came from, for `mt config show --origin`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackOrigin {
    /// A `mindtape:<pack>` entry resolved against [`EMBEDDED_PACKS`], held as
    /// the whole `<source>:<pack>` name.
    Embedded(String),
    /// A path pack naming a TOML file, resolved to an absolute (or as-given,
    /// if the base couldn't be made absolute) path.
    Path(PathBuf),
    /// A directory pack, held as the directory: its `pack.toml` is the pack,
    /// and its `templates/` joins `templates.dirs`. A pack fetched into a
    /// cache resolves to this too.
    Dir(PathBuf),
}

impl PackOrigin {
    /// How an error names the pack: its `<source>:<pack>` name, or its
    /// resolved path.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Embedded(name) => name.clone(),
            Self::Path(path) | Self::Dir(path) => path.display().to_string(),
        }
    }

    /// The pack's directory, when it is a directory pack.
    pub(crate) fn dir(&self) -> Option<&Path> {
        match self {
            Self::Dir(dir) => Some(dir),
            Self::Embedded(_) | Self::Path(_) => None,
        }
    }
}

/// Where one entry in a merged rule array came from: a layer file directly,
/// or a pack that layer's `extends` pulled in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleOrigin {
    /// Declared directly in this config layer.
    Layer(LayerSource),
    /// Pulled in through `extends`.
    Pack(PackOrigin),
}

/// One rule's provenance in a merged array: its report-style label
/// (`tasks.rules.<name>` or `tasks.rules[<index>]`, matching
/// [`crate::rules::RuleSpec::label`]'s own convention) and where it
/// came from. Backs `mt config show --origin`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleOriginEntry {
    /// e.g. `tasks.rules.imperative-title` or `tasks.body.headers.rules[2]`.
    pub label: String,
    /// The pack or layer this entry's current content came from.
    pub origin: RuleOrigin,
    /// The one-based line of the entry's `[[…]]` table header in that source,
    /// or `None` when the source writes the array inline.
    pub line: Option<usize>,
}
