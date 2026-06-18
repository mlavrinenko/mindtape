//! `.mindtape/config.toml` loading and the configurable task schema.
//!
//! The config is a white-list: `MindTape` constrains only what is declared here.
//! Omit a section to leave that aspect unconstrained.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer};
use thiserror::Error;

/// Top-level project configuration.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Task discovery, status lifecycle, and field constraints.
    pub tasks: TasksConfig,
    /// Query defaults.
    pub query: QueryConfig,
}

/// `[tasks]` — where tasks live and how they are constrained.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct TasksConfig {
    /// Directories (relative to the project root) holding one task per file.
    pub roots: Vec<String>,
    /// Status lifecycle.
    pub status: StatusConfig,
    /// Constraints for the configurable enum core fields, keyed by Typst slot
    /// name (`priority`, `grade`, `kind`).
    pub fields: BTreeMap<String, FieldConfig>,
    /// Estimate units allow-list.
    pub estimate: EstimateConfig,
    /// Tag allow-list.
    pub tags: TagsConfig,
    /// Body-shape constraints (heading gating).
    pub body: TaskBodyConfig,
}

impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            roots: vec!["tasks".to_owned()],
            status: StatusConfig::default(),
            fields: BTreeMap::new(),
            estimate: EstimateConfig::default(),
            tags: TagsConfig::default(),
            body: TaskBodyConfig::default(),
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

/// `[tasks.status]` — legal status names and the terminal subset.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct StatusConfig {
    /// Legal status names, in lifecycle order. Empty = unconstrained.
    pub values: Vec<String>,
    /// Subset of `values` that ends a task; required fields are not enforced
    /// once a task is terminal.
    pub terminal: Vec<String>,
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
}

/// Parse a config from a TOML string.
///
/// # Errors
/// Returns [`ConfigError::Parse`] if the TOML is invalid.
pub fn parse_config(text: &str) -> Result<Config, ConfigError> {
    Ok(toml::from_str(text)?)
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
    parse_config(&std::fs::read_to_string(&path)?)
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

/// `[tasks.body.headers]` — heading-presence rules.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct HeadersConfig {
    /// Ordered list of heading gating rules.
    pub rules: Vec<HeaderRuleConfig>,
}

/// One entry in `[[tasks.body.headers.rules]]`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct HeaderRuleConfig {
    /// Path through the heading tree to the target headings.
    pub path: Vec<HeaderPathSegment>,
    /// Whether the target headings must exist or must not.
    pub action: HeaderAction,
    /// Optional filter expression gating this rule to matching tasks only.
    pub when: Option<String>,
    /// Override for the default violation message.
    pub message: Option<String>,
}

/// One step in a heading path.
///
/// - A bare TOML string is an exact, case-sensitive, direct-child match.
/// - `{ match = "regex" }` is a regex match; `recursive = true` finds any descendant.
#[derive(Debug, Clone, PartialEq)]
pub enum HeaderPathSegment {
    /// Exact heading text, direct-child position.
    Exact(String),
    /// Regex heading text pattern.
    Pattern {
        /// Rust regex pattern matched against heading text.
        pattern: String,
        /// Search any descendant (`true`) or only direct children (`false`, default).
        recursive: bool,
    },
}

/// Whether matched headings must exist or must not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderAction {
    /// The path must match at least one heading.
    Require,
    /// The path must match no headings.
    Forbid,
}

impl<'de> Deserialize<'de> for HeaderPathSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SegVisitor;

        impl<'de> serde::de::Visitor<'de> for SegVisitor {
            type Value = HeaderPathSegment;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or `{ match = \"...\", recursive = bool }`")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(HeaderPathSegment::Exact(value.to_owned()))
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut pattern: Option<String> = None;
                let mut recursive = false;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "match" => pattern = Some(map.next_value()?),
                        "recursive" => recursive = map.next_value()?,
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &["match", "recursive"],
                            ));
                        }
                    }
                }
                let pattern = pattern.ok_or_else(|| serde::de::Error::missing_field("match"))?;
                Ok(HeaderPathSegment::Pattern { pattern, recursive })
            }
        }

        deserializer.deserialize_any(SegVisitor)
    }
}

impl<'de> Deserialize<'de> for HeaderAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "require" => Ok(Self::Require),
            "forbid" => Ok(Self::Forbid),
            other => Err(serde::de::Error::custom(format!(
                "invalid action `{other}`; expected `require` or `forbid`"
            ))),
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
