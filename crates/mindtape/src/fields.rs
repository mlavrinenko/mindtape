//! Canonical registry of task fields comparable in a filter expression or a
//! `--sort` spec.
//!
//! `filter::field_value` and `sort`'s generic per-field comparator both read
//! [`FIELDS`], so wiring a field in here gives it filter *and* sort support
//! together — there is no second match statement to remember. Before this
//! module existed, `filter.rs` and `sort.rs` each hand-rolled their own
//! field-name match, and they drifted: `assignee` and `reviewed` were
//! comparable in a filter but silently unsortable.
//!
//! Three fields stay outside this registry because their filter and sort
//! semantics genuinely differ rather than merely being unimplemented:
//! `status` and `ready` (sort orders by legend position / readiness rank, not
//! the field's raw string), and `role` (a virtual, filter-only field with no
//! `Task` field behind it — see `filter::role_field`). `sort.rs` special-cases
//! `status`/`ready`/`priority`/`difficulty` before falling back to this
//! registry for everything else.
//!
//! `path` is the task's project-root-relative source path
//! ([`Task::file_key`]), and is deliberately not spelled `file`: the `file`
//! COLUMN renders the same task's path relative to the working directory, so
//! one name covered two values that coincide inside the project root and
//! diverge under `-w <other-project>` — where a filter copied from the
//! listing matched nothing, silently, an empty listing being also what a
//! correct filter over an empty result set looks like
//! (`truth/tasks/the-file-filter-field-and-the-file-column-name-different-paths.typ`).
//! Display stays working-directory-relative (the only form openable from
//! where the reader is standing); the comparable value stays root-relative,
//! so a filter saved in `.mindtape/config.toml` means the same thing wherever
//! `mt` runs from. [`rename_hint`] carries the old spelling's users across.
//!
//! `depth`, `topo` and `blocks` are *derived* entries: nothing authors them,
//! the loader computes them (folder nesting; dependency level and transitive
//! leverage — see `crate::dep_rank`). They enter here rather than being
//! special-cased because they are plain numbers on `Task` by the time a filter
//! or a sort sees one, so the generic comparator already orders them right —
//! which is the whole point of a registry: a graph-derived rank becomes both
//! `mt ls 'blocks > 3'` and `--sort blocks:desc` by being listed once.

use crate::config::{Config, PriorityConfig};
use crate::model::Task;
use crate::numeric::NumValue;

/// A resolved field value. String fields compare lexicographically
/// (ISO-correct for dates); numeric fields compare as numbers. `Missing`
/// is unequal to every filter literal (see `filter::compare`) and, for sort,
/// orders after every present value.
pub enum FieldValue {
    Str(String),
    Num(f64),
    Missing,
}

/// One comparable field: its filter/`--sort` name and how to read it off a
/// task.
pub struct FieldDef {
    pub name: &'static str,
    pub get: fn(&Task, &Config) -> FieldValue,
}

/// Every generically comparable field. Order is not significant.
pub const FIELDS: &[FieldDef] = &[
    FieldDef {
        name: "status",
        get: |task, _config| opt_str(task.status.as_ref().map(|status| status.name.clone())),
    },
    FieldDef {
        name: "priority",
        get: |task, config| num_field(task.priority.as_ref(), Some(&config.tasks.priority)),
    },
    FieldDef {
        name: "difficulty",
        get: |task, _config| num_field(task.difficulty.as_ref(), None),
    },
    FieldDef {
        name: "kind",
        get: |task, _config| opt_str(task.kind.clone()),
    },
    FieldDef {
        name: "title",
        get: |task, _config| FieldValue::Str(task.title.clone()),
    },
    FieldDef {
        name: "id",
        get: |task, _config| FieldValue::Str(task.id.clone()),
    },
    FieldDef {
        name: "due",
        get: |task, _config| opt_str(task.due.clone()),
    },
    FieldDef {
        name: "start",
        get: |task, _config| opt_str(task.start.clone()),
    },
    FieldDef {
        name: "reviewed",
        get: |task, _config| opt_str(task.reviewed.clone()),
    },
    FieldDef {
        name: "path",
        get: |task, _config| FieldValue::Str(task.file_key()),
    },
    FieldDef {
        name: "assignee",
        get: |task, _config| opt_str(task.assignee.clone()),
    },
    FieldDef {
        name: "depth",
        get: |task, _config| FieldValue::Num(task.depth.into()),
    },
    FieldDef {
        name: "topo",
        get: |task, _config| FieldValue::Num(task.topo.into()),
    },
    FieldDef {
        name: "blocks",
        get: |task, _config| FieldValue::Num(task.blocks.into()),
    },
];

/// Look up a field by its filter/`--sort` name.
#[must_use]
pub fn get(name: &str) -> Option<&'static FieldDef> {
    FIELDS.iter().find(|def| def.name == name)
}

/// Field names this registry has retired, each paired with the name that
/// replaced it and the distinction the rename draws. A rename that only
/// deletes the old name trades a silent wrong answer for a bare "unknown
/// field", which tells the reader they are wrong but not what to type; this
/// table is what [`rename_hint`] turns into the rest of the sentence.
const RENAMED_FIELDS: &[(&str, &str, &str)] = &[(
    "file",
    "path",
    "`file` now names only the ls COLUMN, whose display is relative to the \
     working directory; the comparable field is the root-relative path",
)];

/// The trailing clause an unknown-field error appends when `name` is a
/// retired spelling ([`RENAMED_FIELDS`]) — empty for every other unknown
/// name, so a caller can append it unconditionally.
///
/// Both the filter (`crate::filter::FilterError::UnknownField`) and `--sort`
/// (`crate::sort::validate_sort_spec`) error paths go through this, because
/// the registry backs both and a name retired here is retired in both.
#[must_use]
pub fn rename_hint(name: &str) -> String {
    RENAMED_FIELDS
        .iter()
        .find(|(old, _, _)| *old == name)
        .map_or_else(String::new, |(_, new, why)| {
            format!("; renamed to `{new}` — {why}")
        })
}

/// `body` holds a task's prose, verbatim and already marker-free (the
/// `#show: task.with(…)` head never reaches it). It is deliberately outside
/// [`FIELDS`]: prose is neither comparable (`==`/`<`) nor sortable — the two
/// jobs `FIELDS` backs — so `contains()` is the one place it is searchable.
pub const BODY_FIELD: &str = "body";

/// Every `links[].note` on a task, concatenated for `contains()` matching.
/// Link notes are prose (`related("x")[why]`) but live on `links[].note`, not
/// in [`BODY_FIELD`] — text-only like `body`, so it joins the same tier
/// rather than entering [`FIELDS`].
pub const NOTE_FIELD: &str = "note";

/// A task's authored and derived tags (see `Task::all_tags`), joined for
/// `contains()` matching. `tags` is a controlled vocabulary with `has_tag()`
/// for exact match, so this is the substring-match escape hatch — text-only
/// like `body`, so it joins the same tier rather than entering [`FIELDS`].
pub const TAGS_FIELD: &str = "tags";

/// Fields addressable by `contains()` but carrying no comparison or sort
/// semantics, so they stay out of [`FIELDS`]: prose ([`BODY_FIELD`],
/// [`NOTE_FIELD`]) and the tag vocabulary ([`TAGS_FIELD`]).
const TEXT_ONLY_CONTAINS_FIELDS: [&str; 3] = [BODY_FIELD, NOTE_FIELD, TAGS_FIELD];

/// The `contains()` field vocabulary: every [`FIELDS`] name, plus the
/// text-only tier ([`TEXT_ONLY_CONTAINS_FIELDS`]). One derived source of
/// truth, so `contains()` never hand-matches field names of its own —
/// exactly the drift this module exists to end (see the module doc).
#[must_use]
pub fn is_contains_field(name: &str) -> bool {
    TEXT_ONLY_CONTAINS_FIELDS.contains(&name) || get(name).is_some()
}

/// Every name [`is_contains_field`] accepts, [`FIELDS`] first then the
/// text-only tier — for "expected one of ..." error messages (an unknown
/// explicit `contains()` field, or an unknown name in a configured default
/// haystack).
#[must_use]
pub fn contains_field_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = FIELDS.iter().map(|def| def.name).collect();
    names.extend(TEXT_ONLY_CONTAINS_FIELDS);
    names
}

fn opt_str(value: Option<String>) -> FieldValue {
    value.map_or(FieldValue::Missing, FieldValue::Str)
}

/// A numeric scalar field (`priority`/`difficulty`) as a comparable value: a
/// raw number compares numerically; a label falls back to a string compare
/// (so `priority == "high"` matches the written label); a framework value
/// resolves its score through `priority` when given the priority config.
fn num_field(value: Option<&NumValue>, priority: Option<&PriorityConfig>) -> FieldValue {
    match value {
        Some(NumValue::Num(num)) => FieldValue::Num(*num),
        Some(NumValue::Label(label)) => FieldValue::Str(label.clone()),
        Some(framework @ NumValue::Framework { .. }) => priority
            .and_then(|config| config.resolve(framework))
            .map_or(FieldValue::Missing, FieldValue::Num),
        None => FieldValue::Missing,
    }
}

#[cfg(test)]
#[path = "fields_tests.rs"]
mod tests;
