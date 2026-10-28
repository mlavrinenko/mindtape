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
pub(crate) const EMBEDDED_PACKS: &[(&str, &str)] = &[
    ("tasks", include_str!("../packs/tasks.toml")),
    ("wip-caps", include_str!("../packs/wip-caps.toml")),
    ("budget", include_str!("../packs/budget.toml")),
    ("tweaks", include_str!("../packs/tweaks.toml")),
    ("commits", include_str!("../packs/commits.toml")),
    ("dogfood", include_str!("../packs/dogfood.toml")),
    ("flow", include_str!("../packs/flow.toml")),
    ("ice", include_str!("../packs/ice.toml")),
];

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
    /// and its `templates/` joins `templates.dirs`.
    Dir(PathBuf),
    /// A `<source>:<pack>` entry resolved against a declared source's tree in
    /// the cache: `<pack>.toml` at the tree's top, or `<pack>/pack.toml`.
    Cached {
        /// The whole `<source>:<pack>` name.
        name: String,
        /// The source's tree, `<cache>/<sha256>/`, which every file the pack
        /// names stays inside.
        tree: PathBuf,
        /// The pack's directory, when it is a directory pack.
        dir: Option<PathBuf>,
    },
}

impl PackOrigin {
    /// How an error names the pack: its `<source>:<pack>` name, or its
    /// resolved path.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Embedded(name) | Self::Cached { name, .. } => name.clone(),
            Self::Path(path) | Self::Dir(path) => crate::display_path(path),
        }
    }

    /// The directory the pack's own files resolve from: a directory pack's
    /// directory, or the one holding a pack file. A built-in pack has none.
    pub(crate) fn root(&self) -> Option<&Path> {
        match self {
            Self::Dir(dir) => Some(dir),
            Self::Path(file) => file.parent(),
            Self::Cached { tree, dir, .. } => Some(dir.as_deref().unwrap_or(tree)),
            Self::Embedded(_) => None,
        }
    }

    /// The pack's directory, when it is a directory pack.
    pub(crate) fn dir(&self) -> Option<&Path> {
        match self {
            Self::Dir(dir) => Some(dir),
            Self::Cached { dir, .. } => dir.as_deref(),
            Self::Embedded(_) | Self::Path(_) => None,
        }
    }

    /// The directory every file the pack names must stay inside: a cached
    /// pack's source tree, which came from the network. A local pack may
    /// name any file, as the project itself may.
    pub(crate) fn bound(&self) -> Option<&Path> {
        match self {
            Self::Cached { tree, .. } => Some(tree),
            Self::Embedded(_) | Self::Path(_) | Self::Dir(_) => None,
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
