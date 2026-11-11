//! `[templates]` config: where `mt add` finds task templates.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::config::add_hooks::BUILTIN_TEMPLATE;

/// Where `mt add` finds task templates.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct TemplatesConfig {
    /// Directories, relative to the project root, holding `*.typ` task templates.
    pub dirs: Vec<String>,
    /// Template stem `mt add` uses when none is named.
    pub default: String,
    /// How `mt check` reports a breach of the built-in template, which `mt add`
    /// falls back to when no template dir holds `task.typ`. A project's own
    /// templates always report errors.
    pub fallback_severity: FallbackSeverity,
}

/// How severely a breach of the built-in template is reported.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FallbackSeverity {
    /// Print the breach; `mt check` still succeeds. The default.
    #[default]
    Warning,
    /// Print the breach and fail `mt check`.
    Error,
}

// Left opinionated rather than collapsed to empty, for the same reason as
// `TasksConfig::default`'s `roots`: widely relied on by direct `Config::default()`
// test fixtures (e.g. template resolution/listing tests), and its value already
// agrees with the embedded floor (`defaults.toml`).
impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            dirs: vec![".mindtape/templates".to_owned()],
            default: BUILTIN_TEMPLATE.to_owned(),
            fallback_severity: FallbackSeverity::Warning,
        }
    }
}

impl TemplatesConfig {
    /// The template directories as normalized [`PathBuf`]s, each still
    /// relative to the project root unless it is absolute: `.` components are
    /// dropped and interior `..` is resolved lexically.
    #[must_use]
    pub fn normalized_dirs(&self) -> Vec<PathBuf> {
        (self.dirs.iter())
            .map(|dir| crate::config::roots::normalize_path(Path::new(dir)))
            .collect()
    }
}
