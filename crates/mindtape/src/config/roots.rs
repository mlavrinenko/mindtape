//! Task root specs and the folder-derived-tag policy.
//!
//! `[[tasks.roots]]` entries name where tasks live and, optionally, a display
//! name and per-root derivation rule. `[tasks.derive-tags]` holds the
//! project-wide policy (collectors, separator, slugify). The derivation
//! algorithm itself lives in [`crate::loader::derive`]; this module is config shape only.
//!
//! [`validate_task_sources`] is the one check that refuses a working-tree
//! config whose roots or template dirs can never hold a task source. It reads
//! the normalized forms of [`crate::task_source::TaskSource`], never
//! [`RootSpec::dir`]'s match keys, and names the config key and declaring
//! layer in its error.

use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::config::layers::LayerSource;
use crate::config::pack_registry::RuleOrigin;
use crate::config::{Config, ConfigError};
use crate::task_source::{TaskSource, has_reserved_component};

/// The config key that declares task roots.
const ROOTS_KEY: &str = "tasks.roots";

/// The config key that declares template dirs.
const TEMPLATES_KEY: &str = "templates.dirs";

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

/// Project-wide policy for folder-derived tags.
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
    /// The root directory path, relative to the project root. A path outside
    /// the project (`../shared/tasks`) loads in the working tree only; a
    /// commit state cannot list it.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Short(path) => path,
            Self::Full(cfg) => &cfg.path,
        }
    }

    /// [`Self::path`] normalized for matching a project-relative path: `.`
    /// components and interior `..` are resolved lexically, leading `./` and
    /// trailing `/` are dropped, and `"."` means the project root.
    #[must_use]
    pub fn dir(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        for component in Path::new(self.path()).components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    if parts.last().is_some_and(|part| *part != "..") {
                        parts.pop();
                    } else {
                        parts.push("..");
                    }
                }
                std::path::Component::Normal(name) => {
                    parts.push(name.to_str().unwrap_or_default());
                }
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                    return self.path().to_owned();
                }
            }
        }
        parts.join("/")
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

    /// [`Self::path`] as a normalized [`PathBuf`]: `.` components are dropped
    /// and interior `..` is resolved lexically, without touching the
    /// filesystem. An absolute path stays absolute.
    ///
    /// This is the task-source policy's separate root form;
    /// [`Self::dir`] keeps its string match-key semantics for commit and tag
    /// derivation.
    #[must_use]
    pub fn normalized(&self) -> PathBuf {
        normalize_path(Path::new(self.path()))
    }
}

/// `path` with `.` components dropped and `..` resolved lexically, retaining
/// an absolute path's root. The filesystem is never read.
#[must_use]
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if clean.file_name().is_some() {
                    clean.pop();
                } else if !clean.has_root() {
                    clean.push("..");
                }
            }
            other => clean.push(other.as_os_str()),
        }
    }
    clean
}

fn last_segment(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map_or_else(|| path.to_owned(), |s| s.to_string_lossy().into_owned())
}

/// Refuse a working-tree config whose task roots or template dirs contradict
/// the task-source policy.
///
/// A root inside `.mindtape` or `.git` can never hold a source, and a template
/// dir at or above the project root or a task root would reserve every source
/// under it. Paths are compared in the policy's normalized absolute forms, so
/// `./tasks/../tasks` and `tasks` are one root.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] naming the offending key, path, reason and
/// declaring layer for the first root or template dir that cannot yield a
/// source.
pub(crate) fn validate_task_sources(config: &Config, root: &Path) -> Result<(), ConfigError> {
    let source = TaskSource::new(root, config);
    for (spec, dir) in config.tasks.roots.iter().zip(source.roots()) {
        let task_path = source.task_path(dir);
        if let Some(component) = reserved_component(task_path.rel()) {
            let message = metadata_problem(spec.path(), component);
            return Err(refuse(config, root, ROOTS_KEY, message));
        }
    }
    for (shown, template) in config.templates.dirs.iter().zip(source.templates()) {
        if let Some(message) = template_problem(config, &source, shown, template) {
            return Err(refuse(config, root, TEMPLATES_KEY, message));
        }
    }
    Ok(())
}

/// The `.mindtape` or `.git` component of `path`, in the case its message
/// names it. `None` when the policy sees no reserved component.
fn reserved_component(path: &Path) -> Option<&'static str> {
    if !has_reserved_component(path) {
        return None;
    }
    (path.components()).find_map(|component| match component {
        Component::Normal(name) if name.eq_ignore_ascii_case(".mindtape") => Some(".mindtape"),
        Component::Normal(name) if name.eq_ignore_ascii_case(".git") => Some(".git"),
        _ => None,
    })
}

/// The error text for a root under `component`.
fn metadata_problem(root: &str, component: &str) -> String {
    if component == ".mindtape" {
        format!("{ROOTS_KEY}: `{root}` is reserved for project metadata and templates")
    } else {
        format!("{ROOTS_KEY}: `{root}` is inside `.git`, reserved for git metadata")
    }
}

/// The error text for `template` when it contains the project root or a task
/// root, else `None`.
fn template_problem(
    config: &Config,
    source: &TaskSource,
    shown: &str,
    template: &Path,
) -> Option<String> {
    if source.root().starts_with(template) {
        return Some(format!(
            "{TEMPLATES_KEY}: `{shown}` is at or above the project root, \
             so every task under it would be reserved"
        ));
    }
    let root = contained_root(config, source, template)?;
    Some(format!(
        "{TEMPLATES_KEY}: `{shown}` is at or above task root `{root}`, \
         so every task there would be reserved"
    ))
}

/// The configured task root `template` contains, if any.
fn contained_root<'a>(config: &'a Config, source: &TaskSource, template: &Path) -> Option<&'a str> {
    let index = (source.roots().iter()).position(|root| root.starts_with(template))?;
    config.tasks.roots.get(index).map(RootSpec::path)
}

/// `message` with the layer that set `key`, when the provenance knows it.
fn refuse(config: &Config, root: &Path, key: &str, message: String) -> ConfigError {
    let Some(origin) = config.provenance.key_origin(key) else {
        return ConfigError::invalid(message);
    };
    ConfigError::invalid(format!(
        "{message} (declared in {})",
        origin_label(origin, root)
    ))
}

/// How an error names `origin` to a reader at `root`.
fn origin_label(origin: &RuleOrigin, root: &Path) -> String {
    match origin {
        RuleOrigin::Layer(LayerSource::Embedded) => "the built-in defaults".to_owned(),
        RuleOrigin::Layer(
            LayerSource::Xdg(path) | LayerSource::Dir(path) | LayerSource::Project(path),
        ) => crate::display_path(crate::relative_to(path, root)),
        RuleOrigin::Pack(pack) => pack.label(),
    }
}

#[cfg(test)]
#[path = "roots_tests.rs"]
mod tests;
