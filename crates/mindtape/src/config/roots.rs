//! Task root specs and the folder-derived-tag policy.
//!
//! `[[tasks.roots]]` entries name where tasks live and, optionally, a display
//! name and per-root derivation rule. `[tasks.derive-tags]` holds the
//! project-wide policy (collectors, separator, slugify). The derivation
//! algorithm itself lives in [`crate::loader::derive`]; this module is config shape only.

use std::path::Path;

use serde::Deserialize;

/// One `[[tasks.roots]]` entry: a bare path string, or a table with a root
/// tag name and a per-root tag derivation rule. A list may mix both forms.
// `roots = ["tasks"]` and `[[tasks.roots]]` deserialize to the same
// `Vec<RootSpec>`.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum RootSpec {
    /// Path shorthand, e.g. `"tasks"`.
    Short(String),
    /// Full form: `{ path, name, derive-tags }`.
    Full(RootConfig),
}

/// The table form of a task root.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RootConfig {
    /// Directory (relative to the project root) holding one task per file.
    pub path: String,
    /// Tag emitted for the root itself. Defaults to the path's last segment.
    pub name: Option<String>,
    /// Tag derivation for this root. Absent takes `[tasks.derive-tags] enabled`.
    pub derive_tags: Option<DeriveTags>,
}

/// Per-root tag derivation: a bool, a file glob, or a list of globs. `true` or
/// `false` turns derivation on or off for every file under the root. A glob
/// turns it on only for files whose path, relative to the root, matches.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum DeriveTags {
    /// Enable or disable for every file under the root.
    Enabled(bool),
    /// Enable only for files matching this glob.
    Glob(String),
    /// Enable only for files matching any of these globs.
    Globs(Vec<String>),
}

/// `[tasks.derive-tags]` — project-wide folder-derived-tag policy.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DeriveConfig {
    /// Default derivation switch for roots that set no `derive-tags` of their own.
    pub enabled: bool,
    /// Collectors to run. Both may run at once; their tags are merged
    /// without duplicates.
    pub collect: Vec<Collector>,
    /// Breadcrumb join separator.
    pub separator: String,
    /// Normalize folder names to tag form.
    pub slugify: bool,
    /// Whether breadcrumb tags start with the root name (`mt`, `mt:backend`,
    /// …).
    pub breadcrumb_root: bool,
}

impl Default for DeriveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            collect: vec![Collector::Segments],
            separator: ":".to_owned(),
            slugify: true,
            breadcrumb_root: false,
        }
    }
}

/// A folder-derived-tag collector strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Collector {
    /// Each folder name as an independent flat tag (`backend`, `auth`).
    Segments,
    /// Cumulative path prefixes joined by the separator (`backend`,
    /// `backend:auth`).
    Breadcrumbs,
}

impl RootSpec {
    /// The root directory path, relative to the project root.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Short(path) => path,
            Self::Full(cfg) => &cfg.path,
        }
    }

    /// The tag emitted for the root itself: the configured `name`, else the
    /// path's last segment.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Full(RootConfig {
                name: Some(name), ..
            }) => name.clone(),
            _ => last_segment(self.path()),
        }
    }

    /// The per-root derivation override, if any.
    #[must_use]
    pub fn derive_tags(&self) -> Option<&DeriveTags> {
        match self {
            Self::Full(cfg) => cfg.derive_tags.as_ref(),
            Self::Short(_) => None,
        }
    }
}

fn last_segment(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map_or_else(|| path.to_owned(), |s| s.to_string_lossy().into_owned())
}
