//! `[tasks]` config: the section struct itself, plus the small field types it
//! owns directly and that never grew big enough to earn their own module —
//! priority (numeric field + scoring framework selection), task id format,
//! ad-hoc field constraints, the tag allow-list, and body-shape config.
//!
//! Field types that already outgrew this file live in their own sibling
//! instead and are only referenced here: roots/derive-tags (`config_roots`),
//! the status legend (`config_status`), assignee (`config_assignee`),
//! estimate (`config_estimate`), naming (`crate::naming`), links
//! (`crate::link`), recur (`crate::recur`), limits (`config_limits`), and
//! comments (`config_comments`).

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::body_headers::HeadersConfig;
use crate::config_estimate::EstimateConfig;
use crate::config_required::Requirement;
use crate::config_roots::{DeriveConfig, RootSpec};
use crate::config_status::Legend;
use crate::framework::Framework;
use crate::numeric::NumericField;

/// `[tasks]` — where tasks live and how they are constrained.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TasksConfig {
    /// Directories (relative to the project root) holding one task per file.
    pub roots: Vec<RootSpec>,
    /// Project-wide folder-derived-tag policy.
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
    /// Assignee-field constraints (allowed actors, required).
    pub assignee: crate::config_assignee::AssigneeConfig,
    /// Estimate units allow-list.
    pub estimate: EstimateConfig,
    /// Tag allow-list.
    pub tags: TagsConfig,
    /// Body-shape constraints (heading gating).
    pub body: TaskBodyConfig,
    /// Task id format.
    pub id: IdConfig,
    /// Task stem naming template (`mt add`/`mt recur` minting) and `mt add`'s
    /// default collision policy (`dedup`).
    pub naming: crate::naming::NamingConfig,
    /// Typed relation edge (`links:`) kind registry.
    pub links: crate::link::LinksConfig,
    /// Recurring-tasks feature gate and policy floor (`mt recur`).
    pub recur: crate::recur::RecurConfig,
    /// Work-in-progress caps: how many tasks may match a filter at once
    /// (`[[tasks.limits]]`, enforced by `mt check`).
    pub limits: Vec<crate::config_limits::LimitSpec>,
    /// Review-comment policy over a task's whole source file (`mt check`).
    pub comments: crate::config_comments::CommentsConfig,
}

// `roots` intentionally stays the opinionated `["tasks"]` rather than collapsing
// to empty: `Config::default()` is called directly (bypassing `parse_config`)
// by a wide swath of test fixtures across the workspace that assume a "tasks"
// root exists. Its value already agrees with the embedded floor
// (`defaults.toml`), so the duplication is harmless — `parse_config` never
// observes this impl (the floor always supplies `roots` first).
impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            roots: vec![RootSpec::Short("tasks".to_owned())],
            derive_tags: DeriveConfig::default(),
            status: Legend::default(),
            priority: PriorityConfig::default(),
            difficulty: NumericField::default(),
            fields: BTreeMap::new(),
            assignee: crate::config_assignee::AssigneeConfig::default(),
            estimate: EstimateConfig::default(),
            tags: TagsConfig::default(),
            body: TaskBodyConfig::default(),
            id: IdConfig::default(),
            naming: crate::naming::NamingConfig::default(),
            links: crate::link::LinksConfig::default(),
            recur: crate::recur::RecurConfig::default(),
            limits: Vec::new(),
            comments: crate::config_comments::CommentsConfig::default(),
        }
    }
}

/// `[tasks.priority]` — the priority numeric field plus its scoring frameworks.
///
/// Carries the same scale and band/anchor classification as `difficulty`, plus
/// framework selection on top. When `framework` names an active framework, a
/// task's priority is computed from that framework's inputs instead of a
/// hand-set integer; `allow-manual` decides whether a raw integer is still
/// accepted as an escape hatch. The `frameworks` table holds the built-in
/// frameworks deep-merged with any user-declared ones.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct PriorityConfig {
    /// Scale and band classification (shared numeric-field machinery).
    #[serde(flatten)]
    pub field: NumericField,
    /// Active framework name, matched against `frameworks`. `None` = raw numeric.
    pub framework: Option<String>,
    /// Whether a raw integer is accepted while a framework is active.
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
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
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

/// `[tasks.fields.<name>]` — constraints for one configurable core field.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FieldConfig {
    /// Allowed enum values. Empty = any.
    pub values: Vec<String>,
    /// Default value (informational; not auto-applied during load).
    pub default: Option<String>,
    /// Whether the field must be set.
    pub required: Requirement,
}

/// `[tasks.tags]` — optional tag allow-list.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct TagsConfig {
    /// Allowed tag names. Empty = any.
    pub allowed: Vec<String>,
}

/// `[tasks.body]` — body-shape constraints.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct TaskBodyConfig {
    /// Section-heading gating rules.
    pub headers: HeadersConfig,
}
