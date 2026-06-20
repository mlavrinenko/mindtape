//! `.mindtape/config.toml` loading and the configurable task schema.
//!
//! The config is a white-list: `MindTape` constrains only what is declared here.
//! Omit a section to leave that aspect unconstrained.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer};
use thiserror::Error;

use crate::framework::{Framework, merge_builtin_frameworks};
use crate::numeric::NumericField;

pub use crate::config_roots::{Collector, DeriveConfig, DeriveTags, RootConfig, RootSpec};

/// Top-level project configuration.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Project-level settings (code, etc.).
    pub project: ProjectConfig,
    /// Task discovery, status lifecycle, and field constraints.
    pub tasks: TasksConfig,
    /// Query defaults.
    pub query: QueryConfig,
    /// `rm` (task removal) defaults.
    pub rm: RmConfig,
    /// `add` (task creation) template discovery.
    pub templates: TemplatesConfig,
    /// Typst evaluation settings (local-package overrides).
    pub typst: TypstConfig,
}

/// `[typst]` — Typst evaluation settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct TypstConfig {
    /// Extra `@local/<name>` package overrides, each resolved from an in-tree
    /// path so a worktree resolves a foreign package without a global install.
    /// `MindTape`'s own `@local/mindtape` is auto-registered when `<root>/lib` is
    /// the `mindtape` package; this seam covers any other package (e.g. a
    /// consumer's own lib referenced by an `x-<ns>` extras helper).
    #[serde(rename = "local-packages")]
    pub local_packages: Vec<LocalPackage>,
}

/// `[[typst.local-packages]]` — one `@local/<name>` → in-tree path override.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct LocalPackage {
    /// The `@local/<name>` package name.
    pub name: String,
    /// Lib directory holding the package's `typst.toml`, relative to the root.
    pub path: String,
}

/// `[templates]` — where `mindtape add` finds task templates.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct TemplatesConfig {
    /// Dirs (relative to the project root) holding `*.typ` task templates.
    pub dirs: Vec<String>,
    /// Template stem `mindtape add` uses when none is named.
    pub default: String,
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            dirs: vec![".mindtape/templates".to_owned()],
            default: "task".to_owned(),
        }
    }
}

/// `[project]` — project-level settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProjectConfig {
    /// Short project qualifier. Falls back to the root directory name when absent.
    pub code: Option<String>,
}

/// Resolve the effective project code: configured `code` or the root dir name.
#[must_use]
pub fn effective_project_code(config: &Config, root: &std::path::Path) -> String {
    if let Some(code) = &config.project.code {
        return code.clone();
    }
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `[tasks]` — where tasks live and how they are constrained.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct TasksConfig {
    /// Directories (relative to the project root) holding one task per file.
    pub roots: Vec<RootSpec>,
    /// Project-wide folder-derived-tag policy.
    #[serde(rename = "derive-tags")]
    pub derive_tags: DeriveConfig,
    /// Status legend: legal values in lifecycle order, plus per-value display
    /// (glyph, colour, label) and the `terminal` flag.
    pub status: Legend,
    /// Priority as a numeric-field instance (scale + band classification),
    /// optionally driven by a configured scoring framework.
    pub priority: PriorityConfig,
    /// Difficulty as a numeric-field instance (scale + anchor classification).
    pub difficulty: NumericField,
    /// Constraints for the remaining configurable enum core fields, keyed by
    /// Typst slot name (`kind`).
    pub fields: BTreeMap<String, FieldConfig>,
    /// Estimate units allow-list.
    pub estimate: EstimateConfig,
    /// Tag allow-list.
    pub tags: TagsConfig,
    /// Body-shape constraints (heading gating).
    pub body: TaskBodyConfig,
    /// Task id format.
    pub id: IdConfig,
}

impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            roots: vec![RootSpec::Short("tasks".to_owned())],
            derive_tags: DeriveConfig::default(),
            status: Legend::default(),
            priority: PriorityConfig::default(),
            difficulty: NumericField::default(),
            fields: BTreeMap::new(),
            estimate: EstimateConfig::default(),
            tags: TagsConfig::default(),
            body: TaskBodyConfig::default(),
            id: IdConfig::default(),
        }
    }
}

/// `[tasks.priority]` — the priority numeric field plus its scoring frameworks.
///
/// Flattens a [`NumericField`] (scale + band classification, shared with
/// difficulty) and layers framework selection on top. When `framework` names an
/// active framework, a task's priority is computed from that framework's inputs
/// instead of a hand-set integer; `allow_manual` decides whether a raw integer
/// is still accepted as an escape hatch. The `frameworks` registry holds the
/// baked-in built-ins deep-merged with any user `[tasks.priority.frameworks.*]`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct PriorityConfig {
    /// Scale and band classification (shared numeric-field machinery).
    #[serde(flatten)]
    pub field: NumericField,
    /// Active framework name, matched against `frameworks`. `None` = raw numeric.
    pub framework: Option<String>,
    /// Whether a raw integer is accepted while a framework is active.
    #[serde(rename = "allow-manual")]
    pub allow_manual: bool,
    /// Framework registry: built-ins deep-merged with user definitions.
    pub frameworks: BTreeMap<String, Framework>,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            field: NumericField::default(),
            framework: None,
            allow_manual: true,
            frameworks: BTreeMap::new(),
        }
    }
}

/// `[tasks.id]` — task identity format constraint.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct IdConfig {
    /// Format requirement for an explicit `id:`. Default accepts any non-empty string.
    pub format: IdFormat,
}

/// Format requirement for a task `id:` field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IdFormat {
    /// Any non-empty string (the default).
    #[default]
    String,
    /// A UUID v4 (random).
    UuidV4,
    /// A UUID v7 (time-ordered).
    UuidV7,
}

impl IdFormat {
    /// Generate a fresh id in this format, or `None` when identity is the file
    /// stem (the `String` format carries no generator).
    #[must_use]
    pub fn generate(self) -> Option<String> {
        match self {
            Self::String => None,
            Self::UuidV4 => Some(uuid::Uuid::new_v4().to_string()),
            Self::UuidV7 => Some(uuid::Uuid::now_v7().to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for IdFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "string" => Ok(Self::String),
            "uuid-v4" => Ok(Self::UuidV4),
            "uuid-v7" => Ok(Self::UuidV7),
            other => Err(serde::de::Error::custom(format!(
                "invalid id format `{other}`; expected `string`, `uuid-v4`, or `uuid-v7`"
            ))),
        }
    }
}

/// `[tasks.estimate]` — the estimate units allow-list.
///
/// An estimate is an opaque `unit -> quantity` dict (e.g. `(days: 5, hours: 5)`
/// or `(points: 8)`). Units are free labels, not converted or compared; the
/// config only constrains which unit names are legal. Empty = any unit.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct EstimateConfig {
    /// Allowed unit names. Empty = any unit.
    pub units: Vec<String>,
    /// Whether an estimate must be set.
    pub required: Requirement,
}

/// `[[tasks.status]]` — the status legend: an ordered, displayable vocabulary.
///
/// Each entry binds a value to optional display attributes (glyph, colour,
/// label) plus the `terminal` flag. The legend defines the legal values, their
/// lifecycle order, and how each renders. Empty = unconstrained. The same
/// primitive will serve other discrete enum fields (kind, tags) as they gain
/// display.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Legend {
    /// Ordered entries, in lifecycle / display order.
    pub entries: Vec<LegendEntry>,
}

/// One `[[tasks.status]]` entry: a value plus its display attributes.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct LegendEntry {
    /// The value this entry describes (e.g. a status name).
    pub value: String,
    /// Optional decorative glyph (emoji or ASCII).
    pub glyph: Option<String>,
    /// Optional colour: a named ANSI colour or `#RRGGBB`.
    pub color: Option<String>,
    /// Optional display label; falls back to `value`.
    pub label: Option<String>,
    /// Whether this value ends a task; required fields relax once terminal.
    pub terminal: bool,
}

impl Legend {
    /// Whether the legend constrains nothing (no entries).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether `value` is a legal entry.
    #[must_use]
    pub fn contains(&self, value: &str) -> bool {
        self.entries.iter().any(|entry| entry.value == value)
    }

    /// The entry describing `value`, if any.
    #[must_use]
    pub fn entry(&self, value: &str) -> Option<&LegendEntry> {
        self.entries.iter().find(|entry| entry.value == value)
    }

    /// The values flagged `terminal`.
    #[must_use]
    pub fn terminal(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|entry| entry.terminal)
            .map(|entry| entry.value.clone())
            .collect()
    }
}

/// `[tasks.fields.<name>]` — constraints for one configurable core field.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct FieldConfig {
    /// Allowed enum values. Empty = any.
    pub values: Vec<String>,
    /// Default value (informational; not auto-applied during load).
    pub default: Option<String>,
    /// Whether the field must be set.
    pub required: Requirement,
}

/// `[tasks.tags]` — optional tag allow-list.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct TagsConfig {
    /// Allowed tag names. Empty = any.
    pub allowed: Vec<String>,
}

/// `[query]` — query defaults.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct QueryConfig {
    /// Filter applied by `list` when the caller passes none.
    pub default_filter: Option<String>,
    /// Default sort chain for `list` when the caller passes no `--sort`.
    /// Same spec grammar as `--sort`: `"field"` or `"field:asc"` / `"field:desc"`.
    /// Validated at config load. When absent the shipped built-in chain applies.
    pub default_sort: Vec<String>,
}

/// `[rm]` — defaults for the `rm` task-removal command.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct RmConfig {
    /// Filter selecting which tasks a bare `mindtape rm` (no target) retires.
    /// Built-in default `"terminal"`: sweep every terminal-status task, matching
    /// the legacy `cleanup` behaviour. An explicit ref or filter argument
    /// overrides it.
    pub default_filter: Option<String>,
}

impl Default for RmConfig {
    fn default() -> Self {
        Self {
            default_filter: Some("terminal".to_owned()),
        }
    }
}

/// Whether a field is required: `false`, `true`, or `"non-terminal"`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Requirement {
    /// Never required.
    #[default]
    No,
    /// Always required.
    Always,
    /// Required only while the task is not terminal.
    NonTerminal,
}

impl<'de> Deserialize<'de> for Requirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ReqVisitor;
        impl serde::de::Visitor<'_> for ReqVisitor {
            type Value = Requirement;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a boolean or the string \"non-terminal\"")
            }
            fn visit_bool<E>(self, value: bool) -> Result<Requirement, E> {
                Ok(if value {
                    Requirement::Always
                } else {
                    Requirement::No
                })
            }
            fn visit_str<E>(self, value: &str) -> Result<Requirement, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "non-terminal" => Ok(Requirement::NonTerminal),
                    "true" => Ok(Requirement::Always),
                    "false" => Ok(Requirement::No),
                    other => Err(E::custom(format!("invalid `required` value: {other}"))),
                }
            }
        }
        deserializer.deserialize_any(ReqVisitor)
    }
}

/// Errors from loading or locating a config file.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The config file could not be read.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// The config file contained invalid TOML.
    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
    /// The config parsed but failed a semantic check (e.g. band coverage).
    #[error("invalid config: {0}")]
    Invalid(String),
}

impl Config {
    /// Run semantic checks that go beyond TOML shape (numeric scale and
    /// classification coverage). Returns one joined message per problem.
    ///
    /// # Errors
    /// Returns the collected problems when the numeric fields are inconsistent.
    pub fn validate(&self) -> Result<(), String> {
        let mut problems = self.tasks.priority.validate("priority");
        problems.extend(self.tasks.difficulty.validate("difficulty"));
        problems.extend(crate::derive::validate_root_globs(self));
        for spec in &self.query.default_sort {
            if let Err(msg) = crate::sort::validate_sort_spec(spec) {
                problems.push(format!("query.default_sort: {msg}"));
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems.join("; "))
        }
    }
}

/// Parse a config from a TOML string.
///
/// Built-in priority frameworks are deep-merged into
/// `[tasks.priority.frameworks]` before typed deserialization, so a user config
/// can activate (and selectively override) a built-in without redeclaring it.
///
/// # Errors
/// Returns [`ConfigError::Parse`] if the TOML is invalid.
pub fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let mut doc: toml::Table = toml::from_str(text)?;
    merge_builtin_frameworks(&mut doc);
    Ok(toml::Value::Table(doc).try_into()?)
}

/// Load and parse `<root>/.mindtape/config.toml`, or the defaults if absent.
///
/// # Errors
/// Returns [`ConfigError`] if the file exists but cannot be read or parsed.
pub fn load_config(root: &Path) -> Result<Config, ConfigError> {
    let path = config_path(root);
    if !path.exists() {
        return Ok(Config::default());
    }
    let config = parse_config(&std::fs::read_to_string(&path)?)?;
    config.validate().map_err(ConfigError::Invalid)?;
    Ok(config)
}

/// The conventional config path for a project root.
#[must_use]
pub fn config_path(root: &Path) -> PathBuf {
    root.join(".mindtape").join("config.toml")
}

/// `[tasks.body]` — body-shape constraints.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct TaskBodyConfig {
    /// Section-heading gating rules.
    pub headers: HeadersConfig,
}

pub use crate::body_headers::{HeaderAction, HeaderPathSegment, HeaderRuleConfig, HeadersConfig};

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
