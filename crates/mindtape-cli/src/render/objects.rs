//! The payloads of the object JSON outputs other than `mt ls`: `mt check`,
//! `mt @name check`, `mt graph`, `mt preset ls`, `mt explain` and the two
//! explain listings.
//!
//! Each payload is flattened beside `$schema` by [`crate::render::envelope`],
//! so its keys stay at the top level. One schema comes from each type, so the
//! recorded copy cannot drift from the bytes the binary prints.

use std::collections::BTreeMap;

use mindtape::{ActiveRule, DerivedField};
use serde::Serialize;

use crate::diag::ProblemJson;

/// The document `mt check --json` prints: every problem it found.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CheckPayload {
    /// The problems in print order, each as the text report shows it.
    pub problems: Vec<ProblemJson>,
}

/// The document `mt @name check --json` prints: one report per project.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CheckSetPayload {
    /// The projects' reports, in set order.
    pub projects: Vec<ProjectCheckJson>,
}

/// One project's part of a set check.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ProjectCheckJson {
    /// The project's name, as `mt at <name>` shows it.
    pub project: String,
    /// Whether the project passed.
    pub ok: bool,
    /// How many tasks the project loaded.
    pub tasks: usize,
    /// The problems the project reported.
    pub problems: Vec<ProblemJson>,
}

/// The document `mt graph --json` prints.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GraphPayload {
    /// The tasks the filter selected.
    pub nodes: Vec<NodeJson>,
    /// The typed links among those nodes.
    pub edges: Vec<EdgeJson>,
    /// The links whose kind points outside the backlog.
    pub external: Vec<ExternalJson>,
}

/// One graph node: a task and its readiness.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct NodeJson {
    /// The task's stable id.
    pub id: String,
    /// The task's title.
    pub title: String,
    /// The status name, or `null` when unset.
    pub status: Option<String>,
    /// `ready`, `blocked` or `settled`.
    pub state: String,
}

/// One edge between two graph nodes.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct EdgeJson {
    /// The linking task's id.
    pub from: String,
    /// The linked task's id.
    pub to: String,
    /// The edge kind, such as `depends-on`.
    pub kind: String,
    /// The edge's note, or `null`.
    pub note: Option<String>,
}

/// One link whose kind points outside the backlog.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ExternalJson {
    /// The linking task's id.
    pub from: String,
    /// The edge kind, such as `discharged-by`.
    pub kind: String,
    /// The target as authored, never resolved against the task set.
    pub target: String,
}

/// The document `mt preset ls --json` prints.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct PresetsPayload {
    /// Every built-in, then every user preset.
    pub presets: Vec<PresetJson>,
}

/// One preset, built-in or user.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct PresetJson {
    /// The preset's name.
    pub name: String,
    /// `built-in` or `user`.
    pub origin: String,
    /// The built-in's description; `null` for a user preset.
    pub description: Option<String>,
    /// Whether `mt init` picks this preset with no `-p`.
    pub default: bool,
    /// Present on a user preset that hides a built-in of the same name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadows: Option<bool>,
    /// Present on a built-in a user preset hides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadowed: Option<bool>,
}

/// The document `mt explain --json` prints.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ExplainIndexPayload {
    /// Each config section's rule names, by section path.
    pub rules: BTreeMap<String, Vec<String>>,
    /// Every derived field's name, without the `derived.` prefix.
    pub derived: Vec<String>,
    /// The allowed tags, `default` first.
    pub tags: Vec<String>,
    /// The declared views, `default` first.
    pub views: Vec<String>,
    /// Each dictionary's entry count, `default` first.
    pub dicts: BTreeMap<String, usize>,
}

/// The document `mt explain rules --json` prints.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct RulesPayload {
    /// The rules the config applies, in `mt config show --origin` order.
    pub rules: Vec<ActiveRule>,
    /// The derived fields the config declares.
    pub derived: Vec<DerivedField>,
}

/// The document `mt explain tags --json` prints.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct TagsPayload {
    /// Each allowed tag and the rules that read it.
    pub tags: BTreeMap<String, Vec<TagRuleJson>>,
}

/// One rule that reads a tag.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct TagRuleJson {
    /// The rule's name as `mt config show --origin` prints it.
    pub rule: String,
    /// The message a breach prints.
    pub message: String,
}
