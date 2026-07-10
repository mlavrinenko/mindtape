//! Predicates over a task's typed `links:` edges, split out of `filter.rs`
//! (file-size gate): the `links(kind)` function, and the legacy per-kind field
//! names `has()`/`miss()` still recognise as sugar for it.

use crate::model::Task;

/// The old per-kind field names, still recognised by `has()`/`miss()` as
/// sugar for "this task has at least one edge of the matching kind" —
/// `depends_on`, `supersedes`, `duplicates`. `relates_to` is not carried
/// forward: its replacement kind (`related`) requires a note (the escape
/// hatch), which a presence-only check would silently ignore; reach for
/// `links("related")` instead.
pub(crate) fn sugar_kind(name: &str) -> Option<&'static str> {
    match name {
        "depends_on" => Some("depends-on"),
        "supersedes" => Some("supersedes"),
        "duplicates" => Some("duplicates"),
        _ => None,
    }
}

/// Whether `task` carries at least one `links:` edge of `kind` (built-in or a
/// project's custom kind — this predicate does not validate the name).
pub(crate) fn has_kind(task: &Task, kind: &str) -> bool {
    task.link_targets(kind).next().is_some()
}

#[cfg(test)]
#[path = "filter_links_tests.rs"]
mod tests;
