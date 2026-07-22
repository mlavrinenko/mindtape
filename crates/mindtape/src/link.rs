//! Typed relation edges: the `links:` field's model, kind registry, and config.
//!
//! A task's `links:` field is a flat array of edges, each shaped
//! `(task: "stem.typ", kind: "…", note: none)` by a prelude helper
//! (`depends-on`, `parent`, `recurred-from`, `supersedes`, `duplicates`,
//! `related`, or the generic `edge(kind, ..targets)`). [`Link`] is the parsed
//! Rust projection of one edge; [`parse_links`] projects the whole marker
//! field. Six kinds ship built-in and immutable ([`BUILTIN_KINDS`]); a project
//! or consumer (e.g. speconaut) layers extra kinds on top through
//! `[[tasks.links.kinds]]` ([`LinksConfig`]). [`resolve_kind`] and the
//! `*_kinds` helpers are the single place that merges the two, so validation,
//! readiness, and the dependency graph never hand-roll the lookup.

use serde::Deserialize;

use typst_harvest::HVal;

pub use crate::config_links_validate::TargetValidator;

/// One typed relation edge: a target task, its kind, and an optional note.
///
/// Produced by [`parse_links`] from the `links:` marker field; a bare string
/// entry (no helper used) parses to an empty `kind`, which never matches a
/// built-in or configured kind, so validation rejects it as unknown-kind
/// rather than silently dropping it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Raw target path/stem, as written by the authoring helper (e.g.
    /// `"other.typ"`). Resolved against a loaded task at validation time.
    pub task: String,
    /// Edge kind, matched against [`BUILTIN_KINDS`] then a project's
    /// `[[tasks.links.kinds]]`. Empty when the marker held a bare string.
    pub kind: String,
    /// Free-form note; required, optional, or forbidden depending on the
    /// kind's [`NoteRule`].
    pub note: Option<String>,
}

/// Parse the `links:` marker field into edges. History-pointer targets
/// (SWHID tombstones minted by `cleanup` in place of a retired task) are
/// excluded — they name no loaded task, so they are not a task↔task edge; see
/// [`crate::model::is_history_pointer`].
#[must_use]
pub fn parse_links(value: Option<&HVal>) -> Vec<Link> {
    value
        .and_then(HVal::as_array)
        .map(|items| items.iter().filter_map(parse_one).collect())
        .unwrap_or_default()
}

/// Parse one `links:` entry: the helper-shaped dict (`task`, `kind`, `note`),
/// or a bare string (kept with an empty `kind` so validation can flag it — see
/// [`Link::kind`]). Anything else (a stray non-string, non-dict value) is
/// dropped; the prelude never emits one.
fn parse_one(item: &HVal) -> Option<Link> {
    let (task, kind, note) = if let Some(dict) = item.as_dict() {
        (
            dict.get("task")
                .and_then(HVal::as_str)
                .unwrap_or_default()
                .to_owned(),
            dict.get("kind")
                .and_then(HVal::as_str)
                .unwrap_or_default()
                .to_owned(),
            dict.get("note")
                .and_then(HVal::as_str)
                .map(ToOwned::to_owned),
        )
    } else {
        (item.as_str()?.to_owned(), String::new(), None)
    };
    if crate::model::is_history_pointer(&task) {
        return None;
    }
    Some(Link { task, kind, note })
}

/// A resolved per-kind rule: cardinality cap, note requirement, and whether
/// the kind gates readiness / must stay acyclic. Built by merging
/// [`BUILTIN_KINDS`] with a project's custom `[[tasks.links.kinds]]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindRule {
    /// Cardinality cap (max edges of this kind per task); `None` = unbounded.
    pub max: Option<u32>,
    /// Note requirement.
    pub note: NoteRule,
    /// Whether edges of this kind gate readiness (`ready`/`blocked`).
    pub blocking: bool,
    /// Whether edges of this kind must stay acyclic.
    pub acyclic: bool,
}

/// The six built-in edge kinds, immutable and baked into code: a project's
/// config can only add kinds, never override or remove one of these (see
/// [`resolve_kind`], which always checks this table first).
const BUILTIN_KINDS: &[(&str, KindRule)] = &[
    (
        "depends-on",
        KindRule {
            max: None,
            note: NoteRule::Optional,
            blocking: true,
            acyclic: true,
        },
    ),
    (
        "parent",
        KindRule {
            max: Some(1),
            note: NoteRule::Optional,
            blocking: false,
            acyclic: true,
        },
    ),
    (
        "recurred-from",
        KindRule {
            max: Some(1),
            note: NoteRule::Optional,
            blocking: false,
            acyclic: false,
        },
    ),
    (
        "supersedes",
        KindRule {
            max: None,
            note: NoteRule::Optional,
            blocking: false,
            acyclic: false,
        },
    ),
    (
        "duplicates",
        KindRule {
            max: None,
            note: NoteRule::Optional,
            blocking: false,
            acyclic: false,
        },
    ),
    (
        "related",
        KindRule {
            max: None,
            note: NoteRule::Required,
            blocking: false,
            acyclic: false,
        },
    ),
];

/// Whether `name` is one of the immutable built-in kinds.
#[must_use]
pub fn is_builtin_kind(name: &str) -> bool {
    BUILTIN_KINDS.iter().any(|(builtin, _)| *builtin == name)
}

/// Resolve `name`'s effective rule: a built-in first (immutable — never
/// shadowed by a same-named config entry), else a project's custom
/// `[[tasks.links.kinds]]`. `None` when `name` is neither — the unknown-kind
/// case validation rejects.
#[must_use]
pub fn resolve_kind(name: &str, config: &LinksConfig) -> Option<KindRule> {
    if let Some((_, rule)) = BUILTIN_KINDS.iter().find(|(builtin, _)| *builtin == name) {
        return Some(*rule);
    }
    config
        .kinds
        .iter()
        .find(|k| k.name == name)
        .map(|k| KindRule {
            max: k.max,
            note: k.note,
            blocking: k.blocking,
            acyclic: k.acyclic,
        })
}

/// Whether `name`'s targets are local task files — the axis
/// `[[tasks.links.kinds]] validate` sets. Built-ins always are (their validator
/// is fixed at `local-task-path`), and an unknown kind is treated as one so it
/// still reports as unresolved-and-unknown rather than silently skipping
/// resolution. Anything else names an artifact outside the backlog, so it takes
/// no part in stem resolution, cardinality, cycle detection, or readiness.
#[must_use]
pub fn kind_targets_tasks(name: &str, config: &LinksConfig) -> bool {
    if is_builtin_kind(name) {
        return true;
    }
    config
        .kinds
        .iter()
        .find(|kind| kind.name == name)
        .is_none_or(|kind| kind.validate.targets_tasks())
}

/// Kind names (built-in ∪ config) whose [`KindRule`] satisfies `pick`, built-ins
/// first, each name at most once. A custom kind whose targets are not local
/// tasks never matches: `blocking`/`acyclic` are statements about the task
/// graph, and such a kind's edges are not in it.
fn kinds_matching(config: &LinksConfig, pick: impl Fn(&KindRule) -> bool) -> Vec<&str> {
    let mut kinds: Vec<&str> = BUILTIN_KINDS
        .iter()
        .filter(|(_, rule)| pick(rule))
        .map(|(name, _)| *name)
        .collect();
    for custom in &config.kinds {
        let rule = KindRule {
            max: custom.max,
            note: custom.note,
            blocking: custom.blocking,
            acyclic: custom.acyclic,
        };
        if custom.validate.targets_tasks() && pick(&rule) && !kinds.contains(&custom.name.as_str())
        {
            kinds.push(custom.name.as_str());
        }
    }
    kinds
}

/// Kind names (built-in ∪ config) that gate readiness. Only `depends-on` ships
/// flagged this way; a custom kind can opt in via `blocking = true`.
#[must_use]
pub fn blocking_kinds(config: &LinksConfig) -> Vec<&str> {
    kinds_matching(config, |rule| rule.blocking)
}

/// Kind names (built-in ∪ config) that must stay acyclic. `depends-on` and
/// `parent` ship flagged this way; a custom kind can opt in via `acyclic = true`.
#[must_use]
pub fn acyclic_kinds(config: &LinksConfig) -> Vec<&str> {
    kinds_matching(config, |rule| rule.acyclic)
}

/// `[tasks.links]` — the typed-edge kind registry.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinksConfig {
    /// User-declared custom edge kinds, layered over the immutable built-ins.
    pub kinds: Vec<LinkKind>,
}

/// `[[tasks.links.kinds]]` — one project/consumer-declared custom edge kind.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinkKind {
    /// Kind name, matched against an edge's `kind`.
    pub name: String,
    /// Cardinality cap; `None` = unbounded.
    pub max: Option<u32>,
    /// Note requirement for edges of this kind.
    pub note: NoteRule,
    /// Whether edges of this kind gate readiness. Ignored unless `validate` is
    /// `local-task-path` — a target outside the backlog is not a task to wait on.
    pub blocking: bool,
    /// Whether edges of this kind must stay acyclic. Ignored unless `validate`
    /// is `local-task-path` — there is no cycle without a task-graph edge.
    pub acyclic: bool,
    /// What shape this kind's targets take. Defaults to `local-task-path`,
    /// which is exactly the historical behaviour (a stem resolved against the
    /// loaded tasks); any other value declares the targets to be artifacts
    /// outside the backlog — a commit, a pull request, a build.
    pub validate: TargetValidator,
    /// Pattern for `validate = "regex"`: required for that validator, and
    /// rejected for every other one. A missing or uncompilable pattern is a
    /// config error, never a silently permissive kind.
    pub regex: Option<String>,
}

impl Default for LinkKind {
    fn default() -> Self {
        Self {
            name: String::new(),
            max: None,
            note: NoteRule::Optional,
            blocking: false,
            acyclic: false,
            validate: TargetValidator::LocalTaskPath,
            regex: None,
        }
    }
}

/// Note requirement for an edge kind.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NoteRule {
    /// A non-empty note is mandatory (the `related` escape hatch).
    Required,
    /// A note is allowed but not required (the default).
    #[default]
    Optional,
    /// A note is rejected.
    Forbidden,
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
