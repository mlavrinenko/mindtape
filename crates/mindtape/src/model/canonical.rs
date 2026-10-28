//! The canonical task-JSON schema: the schema of record shared by `mt ls
//! --json` (serialize, via [`CanonicalTask::from_task`]) and `mt import`
//! (deserialize). One shape, so the two never drift: every field below
//! round-trips.
//!
//! # Field contract
//!
//! Every [`CanonicalTask`] field is either *authored* (round-trips through
//! `mt import`; the importer reads it back onto a [`Task`]) or *computed*
//! (read-only; present for convenience/inspection, ignored on import):
//!
//! | Field           | Kind     | Notes                                             |
//! |-----------------|----------|----------------------------------------------------|
//! | `id`            | computed | defaults to the file stem                          |
//! | `file`          | computed | project-relative path                               |
//! | `title`         | authored |                                                      |
//! | `status`        | authored | `{name, date, note}`, null when unset               |
//! | `priority`      | authored | see below                                           |
//! | `difficulty`    | authored | number, label string, or framework-name fallback    |
//! | `kind`          | authored |                                                      |
//! | `assignee`      | authored |                                                      |
//! | `estimate`      | authored | opaque `unit -> quantity` map; a `count(...)` unit holds its resolved number |
//! | `estimate_counts` | authored | `count(...)` units as `{count, queries, problem}`; import reads `count` and `queries` |
//! | `links`         | authored | `[{task, kind, note}]`                              |
//! | `waits_on`      | authored | `checker:id` entries                                |
//! | `tags`          | authored | authored tags only                                  |
//! | `derived_tags`  | computed | folder-derived virtual tags; import reads `tags` only |
//! | `why`           | computed | unfinished blocking targets `[{stem, status, kind}]`; empty unless blocked |
//! | `children`      | computed | count of tasks whose `parent` edge names this one  |
//! | `progress`      | computed | terminal children / children, `0..1`; null with no children |
//! | `due`           | authored |                                                      |
//! | `start`         | authored |                                                      |
//! | `reviewed`      | authored |                                                      |
//! | `body`          | authored | raw Typst source after `#show: task.with(...)`       |
//! | `recur`         | authored | `{rule, anchor, until, count, materialize}`, null when the task does not recur |
//! | `extras`        | authored | opaque `x-<ns>` consumer extension slots             |
//!
//! ## Priority parity
//!
//! Priority emits its *authored* form, not just the resolved score, so the
//! importer can rebuild the exact `priority: ice(...)` call:
//!
//! - A framework value (`NumValue::Framework { name, inputs }`) becomes an
//!   object: `{"framework": "ice", "inputs": {"impact": 8, ...}, "score": 6}`.
//!   `score` is the read-side computed extra (`config.tasks.priority.resolve`),
//!   nullable when unresolvable (e.g. no project config in scope). The
//!   importer reads `framework`/`inputs` and ignores `score`.
//! - A scalar or label value stays a **bare number or string** (e.g.
//!   `"priority": 8`), matching `mt ls --json`'s historical shape — not an
//!   `{"value": ...}` wrapper. This is the one place the schema is
//!   shape-polymorphic; `difficulty` mirrors the non-framework arm (a
//!   framework difficulty, structurally possible but never authored in
//!   practice, falls back to its framework name as a label string).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use typst_harvest::HVal;

use crate::config::Config;
use crate::estimate_queries::{CountEstimate, Query};
use crate::link::Link;
use crate::model::numeric::{InputValue, NumValue};
use crate::model::{Status, Task};

/// Schema version for [`CanonicalTask`]. Bump when the JSON shape changes in
/// a way that could break `mt import` or an external `jq` consumer.
pub const SCHEMA_VERSION: u32 = 1;

/// One task, projected into the canonical JSON schema. See the module doc for
/// the authored/computed field contract.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CanonicalTask {
    /// Stable identity: the file stem, or an explicit authored `id:`.
    #[serde(default)]
    pub id: String,
    /// Path to the source file, relative to the project root and
    /// `/`-separated on every platform ([`Task::file_key`]).
    #[serde(default)]
    pub file: String,
    /// Human-readable title.
    pub title: String,
    /// Lifecycle status, or `null` when unset.
    pub status: Option<CanonicalStatus>,
    /// Priority: authored framework object, or a bare scalar/label. See the
    /// module doc's "Priority parity" section.
    pub priority: Option<CanonicalPriority>,
    /// Difficulty: a bare number or label string.
    pub difficulty: Option<CanonicalNum>,
    /// Kind enum value.
    pub kind: Option<String>,
    /// The actor currently working the task.
    pub assignee: Option<String>,
    /// Opaque `unit -> quantity` estimate map.
    #[serde(default)]
    pub estimate: BTreeMap<String, f64>,
    /// The units written as `count(...)`, in source form. On import a unit
    /// here wins over its resolved number in [`Self::estimate`].
    #[serde(default)]
    pub estimate_counts: BTreeMap<String, CanonicalCount>,
    /// Typed `links:` edges.
    #[serde(default)]
    pub links: Vec<CanonicalLink>,
    /// What the task waits on outside the repo, as `checker:id` entries.
    #[serde(default)]
    pub waits_on: Vec<String>,
    /// Authored tags, exactly as written in the file.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Folder-derived virtual tags (read-only; never authored).
    #[serde(default)]
    pub derived_tags: Vec<String>,
    /// What the task waits on: its unfinished direct blocking-kind targets
    /// (read-only). Needs the whole task set, so [`Self::from_task`] leaves
    /// it empty and a listing fills it from its [`crate::Readiness`].
    #[serde(default)]
    pub why: Vec<crate::graph::ready::Blocker>,
    /// How many tasks name this one with a `parent` edge (read-only).
    #[serde(default)]
    pub children: u32,
    /// Terminal children over all children, `0..1` — the fraction the filter
    /// language compares, where the table renders `3/5`; `null` with no
    /// children (read-only).
    #[serde(default)]
    pub progress: Option<f64>,
    /// Due date, `YYYY-MM-DD`.
    pub due: Option<String>,
    /// Start date, `YYYY-MM-DD`.
    pub start: Option<String>,
    /// Last re-affirmation date, `YYYY-MM-DD`.
    pub reviewed: Option<String>,
    /// Raw Typst source after the `#show: task.with(...)` marker.
    #[serde(default)]
    pub body: String,
    /// Recurrence rule + policy (`every(...)`/`rrule(...)`), or `null` when
    /// the task does not recur. Additive relative to `SCHEMA_VERSION` 1 (an
    /// absent key parses to `None` via `#[serde(default)]`, and no consumer
    /// reading the earlier field set can trip over a new one appearing) —
    /// see `mindtape-cli::output::truncated_to_json`'s "additive, no
    /// `SCHEMA_VERSION` bump" precedent for envelope fields; this is the
    /// same rule applied to a task field.
    #[serde(default)]
    pub recur: Option<crate::recur::CanonicalRecur>,
    /// Opaque consumer extensions, keyed by `x-<ns>` slot.
    #[serde(default)]
    pub extras: BTreeMap<String, HVal>,
}

/// `status:` in the canonical schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalStatus {
    /// Lifecycle name, e.g. `proposed`, `wip`, `done`.
    pub name: String,
    /// Date the status was set, `YYYY-MM-DD`.
    pub date: Option<String>,
    /// Free-form note.
    pub note: Option<String>,
}

/// `links:` edge in the canonical schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalLink {
    /// Raw target path/stem, as authored.
    pub task: String,
    /// Edge kind (e.g. `depends-on`, `related`).
    pub kind: String,
    /// Free-form note; kind-dependent requirement.
    pub note: Option<String>,
}

/// A plain numeric-field value in the canonical schema: a scale point or a
/// band/anchor label. Used for `difficulty`, and for `priority`'s
/// non-framework arm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CanonicalNum {
    /// A raw number on the scale.
    Num(f64),
    /// A band/anchor label, or a framework's bare name (fallback only).
    Label(String),
}

/// `priority:` in the canonical schema: either a plain numeric-field value
/// (see [`CanonicalNum`]) or an authored framework invocation. See the module
/// doc's "Priority parity" section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CanonicalPriority {
    /// A raw number on the scale.
    Num(f64),
    /// A band/anchor label.
    Label(String),
    /// An authored priority-framework invocation.
    Framework {
        /// Framework name, matched against the configured registry.
        framework: String,
        /// Named inputs, as authored.
        #[serde(default)]
        inputs: BTreeMap<String, Value>,
        /// Read-side computed score (`config.tasks.priority.resolve`);
        /// `null`/absent when unresolvable. Ignored on import.
        #[serde(default)]
        score: Option<f64>,
    },
}

/// One `count(...)` estimate unit: its manual part, its queries, and why it
/// didn't resolve (read-only, `null` once it did).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CanonicalCount {
    /// The manual part, `n`.
    pub count: f64,
    /// The queries, in authored order.
    #[serde(default)]
    pub queries: Vec<CanonicalQuery>,
    /// Why the unit didn't resolve (read-only).
    #[serde(default)]
    pub problem: Option<String>,
}

/// One query of a [`CanonicalCount`], tagged by `query`. `files` is how many
/// files it matched (read-only), absent before the unit resolves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "query", rename_all = "lowercase")]
pub enum CanonicalQuery {
    /// `grep(regex, glob)`.
    Grep {
        /// The pattern.
        regex: String,
        /// The optional glob or `@name` set.
        #[serde(default)]
        glob: Option<String>,
        /// Files matched (read-only).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        files: Option<usize>,
    },
    /// `glob(pattern)`.
    Glob {
        /// The glob or `@name` set.
        pattern: String,
        /// Files matched (read-only).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        files: Option<usize>,
    },
}

impl CanonicalCount {
    /// `count` in canonical form, each query beside the files it matched.
    #[must_use]
    pub fn from_count(count: &CountEstimate) -> Self {
        let queries = count
            .queries
            .iter()
            .enumerate()
            .map(|(index, query)| {
                let files = count.matches.get(index).copied();
                match query.clone() {
                    Query::Grep { regex, glob } => CanonicalQuery::Grep { regex, glob, files },
                    Query::Glob { pattern } => CanonicalQuery::Glob { pattern, files },
                }
            })
            .collect();
        Self {
            count: count.manual,
            queries,
            problem: count.problem.clone(),
        }
    }

    /// The authored half, as the task model holds it before it resolves.
    #[must_use]
    pub fn to_count(&self) -> CountEstimate {
        let queries = self
            .queries
            .iter()
            .map(|query| match query.clone() {
                CanonicalQuery::Grep { regex, glob, .. } => Query::Grep { regex, glob },
                CanonicalQuery::Glob { pattern, .. } => Query::Glob { pattern },
            })
            .collect();
        CountEstimate {
            manual: self.count,
            queries,
            matches: Vec::new(),
            problem: None,
        }
    }
}

impl CanonicalTask {
    /// Project `task` into the canonical schema. `config` resolves a
    /// framework priority's read-side `score`.
    #[must_use]
    pub fn from_task(task: &Task, config: &Config) -> Self {
        Self {
            id: task.id.clone(),
            file: task.file_key(),
            title: task.title.clone(),
            status: task.status.as_ref().map(canonical_status),
            priority: priority_to_canonical(task.priority.as_ref(), config),
            difficulty: num_to_canonical(task.difficulty.as_ref()),
            kind: task.kind.clone(),
            assignee: task.assignee.clone(),
            estimate: task.estimate.clone(),
            estimate_counts: task
                .estimate_counts
                .iter()
                .map(|(unit, count)| (unit.clone(), CanonicalCount::from_count(count)))
                .collect(),
            links: task.links.iter().map(canonical_link).collect(),
            waits_on: task.waits_on.clone(),
            tags: task.tags.clone(),
            derived_tags: task.derived_tags.clone(),
            why: Vec::new(),
            children: task.children,
            progress: task.progress(),
            due: task.due.clone(),
            start: task.start.clone(),
            reviewed: task.reviewed.clone(),
            body: task.body.clone(),
            recur: task.recur.as_ref().map(crate::recur::Recur::to_canonical),
            extras: task.extras.clone(),
        }
    }
}

fn canonical_status(status: &Status) -> CanonicalStatus {
    CanonicalStatus {
        name: status.name.clone(),
        date: status.date.clone(),
        note: status.note.clone(),
    }
}

fn canonical_link(link: &Link) -> CanonicalLink {
    CanonicalLink {
        task: link.task.clone(),
        kind: link.kind.clone(),
        note: link.note.clone(),
    }
}

/// Project a numeric-field value into its plain canonical form: a framework
/// value falls back to its name (framework priorities route through
/// [`priority_to_canonical`] instead; this arm only fires for a structurally
/// possible but never-authored framework difficulty).
fn num_to_canonical(value: Option<&NumValue>) -> Option<CanonicalNum> {
    value.map(|value| match value {
        NumValue::Num(num) => CanonicalNum::Num(*num),
        NumValue::Label(label) => CanonicalNum::Label(label.clone()),
        NumValue::Framework { name, .. } => CanonicalNum::Label(name.clone()),
    })
}

/// Project a priority value into its canonical form: a framework value
/// becomes the authored `{framework, inputs, score}` object (`score` is the
/// read-side extra, resolved via `config`); else the plain scalar/label form.
fn priority_to_canonical(value: Option<&NumValue>, config: &Config) -> Option<CanonicalPriority> {
    let value = value?;
    Some(match value {
        NumValue::Framework { name, inputs } => CanonicalPriority::Framework {
            framework: name.clone(),
            inputs: inputs
                .iter()
                .map(|(key, val)| (key.clone(), input_value_to_json(val)))
                .collect(),
            score: config.tasks.priority.resolve(value),
        },
        NumValue::Num(num) => CanonicalPriority::Num(*num),
        NumValue::Label(label) => CanonicalPriority::Label(label.clone()),
    })
}

fn input_value_to_json(value: &InputValue) -> Value {
    match value {
        InputValue::Num(num) => Value::from(*num),
        InputValue::Bool(flag) => Value::from(*flag),
        InputValue::Text(text) => Value::from(text.clone()),
    }
}

#[cfg(test)]
#[path = "canonical_tests.rs"]
mod tests;
