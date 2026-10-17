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
//!
//! # Files
//!
//! - `scan.rs`: the parse-plane scan for a task's outbound link targets.

pub mod scan;

use serde::Deserialize;

use typst_harvest::HVal;

pub use crate::config::links_validate::TargetValidator;

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
/// [`crate::model::is_history_pointer`]. A present-but-not-an-array value (a
/// bare string, say) reports a [`crate::model::marker::ShapeError`] into `errors`
/// rather than silently coercing to no edges.
///
/// A nested entry is flattened rather than dropped: each prelude helper returns
/// an array, so `links: (depends-on("a.typ"), depends-on("b.typ"))` — the array
/// spelling of the `+` concatenation — arrives as an array of arrays. The
/// prelude normalizes that at the source, but the prelude ships as a versioned
/// Typst package installed per machine (`mt install`), so a project still
/// resolving an older copy emits the nested shape to a current binary; this
/// keeps its edges rather than losing half a dependency graph to a version
/// skew. Anything that is neither an edge nor a nest — a number, a date —
/// reports a [`ShapeError`](crate::model::marker::ShapeError) naming its position,
/// closing the last silent drop on this field.
#[must_use]
pub fn parse_links(
    value: Option<&HVal>,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) -> Vec<Link> {
    let Some(value) = value else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        errors.push(crate::model::marker::shape_error(
            "links", "an array", value,
        ));
        return Vec::new();
    };
    let mut links = Vec::new();
    collect_links(items, &mut Vec::new(), &mut links, errors);
    links
}

/// Append every edge in `items` to `out`, descending into a nested array and
/// reporting anything else. `path` is the entry's index trail from the `links:`
/// field down (`[0]`, `[1][0]`), so a report names the offending position in a
/// shape the author can count to in their own source.
fn collect_links(
    items: &[HVal],
    path: &mut Vec<usize>,
    out: &mut Vec<Link>,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) {
    for (index, item) in items.iter().enumerate() {
        path.push(index);
        if let Some(nested) = item.as_array() {
            collect_links(nested, path, out, errors);
        } else if item.as_dict().is_some() || item.as_str().is_some() {
            out.extend(parse_one(item));
        } else {
            errors.push(crate::model::marker::shape_error(
                &format!("links{}", index_path(path)),
                "an edge like depends-on(\"other.typ\")",
                item,
            ));
        }
        path.pop();
    }
}

/// An index trail rendered as Typst-ish subscripts: `[0]`, `[1][0]`.
fn index_path(path: &[usize]) -> String {
    path.iter().map(|index| format!("[{index}]")).collect()
}

/// Parse one `links:` entry: the helper-shaped dict (`task`, `kind`, `note`),
/// or a bare string (kept with an empty `kind` so validation can flag it — see
/// [`Link::kind`]). `None` only for a history pointer; a wrong-shaped entry
/// never reaches here (see [`collect_links`]).
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

/// The built-in kinds with their rules, in table order: the list `mt check`
/// resolves a kind against first, which `mt _cli --json` hands the docs.
pub fn builtin_kinds() -> impl Iterator<Item = (&'static str, KindRule)> {
    BUILTIN_KINDS.iter().copied()
}

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

/// Every kind name this project accepts: the six built-ins first, then its
/// `[[tasks.links.kinds]]` additions, each name at most once.
///
/// [`resolve_kind`] answers one name at a time, which is all validation needs;
/// a *writer* rejecting a kind before it writes needs the enumeration instead —
/// to list what is configured, and to feed [`crate::suggest`] a candidate set.
/// Unlike [`kinds_matching`], this keeps a non-task kind: `mt link` can write
/// an `edge("discharged-by", …)` as legitimately as a `depends-on`.
#[must_use]
pub fn kind_names(config: &LinksConfig) -> Vec<&str> {
    let mut names: Vec<&str> = BUILTIN_KINDS.iter().map(|(name, _)| *name).collect();
    for custom in &config.kinds {
        if !names.contains(&custom.name.as_str()) {
            names.push(custom.name.as_str());
        }
    }
    names
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

/// `[tasks.links]` — the link kinds a task's `links:` edges may use.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinksConfig {
    /// Custom link kinds, added to the built-in ones. A built-in kind cannot
    /// be redefined.
    pub kinds: Vec<LinkKind>,
}

/// `[[tasks.links.kinds]]` — one custom link kind.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinkKind {
    /// Kind name, matched against an edge's `kind`.
    pub name: String,
    /// Most edges of this kind one task may carry. Absent means no limit.
    pub max: Option<u32>,
    /// Note requirement for edges of this kind.
    pub note: NoteRule,
    /// Whether an unfinished target keeps the task from being ready. Applies
    /// only when `validate` is `local-task-path`, because only a task can be
    /// unfinished.
    pub blocking: bool,
    /// Whether edges of this kind must not form a cycle. Applies only when
    /// `validate` is `local-task-path`, because only edges between tasks can
    /// form one.
    pub acyclic: bool,
    /// What this kind's targets are. The default, `local-task-path`, is a
    /// path from the linking task's file to another task's file. Any other value makes the targets something outside the
    /// backlog, such as a commit, a pull request or a build.
    pub validate: TargetValidator,
    /// Regex a target must match when `validate = "regex"`. That validator
    /// requires it, and every other validator rejects it. A missing or invalid
    /// pattern is a config error.
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
    /// A non-empty note is required, as on the built-in `related` kind.
    Required,
    /// A note is allowed but not required (the default).
    #[default]
    Optional,
    /// A note is rejected.
    Forbidden,
}

#[cfg(test)]
mod tests;
