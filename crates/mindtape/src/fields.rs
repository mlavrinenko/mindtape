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
        name: "file",
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
];

/// Look up a field by its filter/`--sort` name.
#[must_use]
pub fn get(name: &str) -> Option<&'static FieldDef> {
    FIELDS.iter().find(|def| def.name == name)
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
