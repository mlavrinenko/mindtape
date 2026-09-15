//! Predicates over a task's typed `links:` edges, split out of `filter.rs`
//! (file-size gate): the `links(kind)` function.

use crate::model::Task;

/// Whether `task` carries at least one `links:` edge of `kind` (built-in or a
/// project's custom kind — this predicate does not validate the name).
pub(crate) fn has_kind(task: &Task, kind: &str) -> bool {
    task.link_targets(kind).next().is_some()
}

#[cfg(test)]
#[path = "filter_links_tests.rs"]
mod tests;
