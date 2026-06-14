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
    /// Constraints for the configurable core fields, keyed by Typst slot name
    /// (`priority`, `grade`, `kind`, `estimate-hours`).
    pub fields: BTreeMap<String, FieldConfig>,
    /// Tag allow-list.
    pub tags: TagsConfig,
}

impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            roots: vec!["tasks".to_owned()],
            status: StatusConfig::default(),
            fields: BTreeMap::new(),
            tags: TagsConfig::default(),
        }
    }
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
    /// Field type; `"number"` enables `min`/`max` bounds.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Inclusive lower bound for numeric fields.
    pub min: Option<f64>,
    /// Inclusive upper bound for numeric fields.
    pub max: Option<f64>,
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

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
