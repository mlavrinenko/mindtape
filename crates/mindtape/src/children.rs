//! A task's children: the tasks whose `parent` edge names it.
//!
//! One definition of the edge feeds three readers, so they never disagree about
//! who a parent's children are: the derived `children`/`progress` fields
//! ([`crate::dep_rank::annotate`]), the `mt flip` refusal to close a parent over
//! unfinished children, and the `mt check` warning about one already closed.
//! "Unfinished" is non-terminal, the same line [`crate::ready`] draws for a
//! blocker: a cancelled child is finished, since nothing is left to wait for.
//!
//! A `parent` target naming no loaded task is not an edge, and a child listing
//! the same parent twice counts once.

use std::collections::HashMap;
use std::path::Path;

use crate::config::Config;
use crate::graph::DepIndex;
use crate::model::Task;
use crate::ready::{Blocker, list_targets};
use crate::severity::Severity;
use crate::validate::Violation;

/// The built-in edge kind a child names its parent with.
pub(crate) const PARENT: &str = "parent";

/// Every loaded parent's children, keyed by the parent's [`Task::file`], each
/// list in `tasks` order and free of duplicates.
pub(crate) fn children_by_parent<'a>(tasks: &'a [Task]) -> HashMap<&'a Path, Vec<&'a Task>> {
    let index = DepIndex::build(tasks);
    let mut out: HashMap<&'a Path, Vec<&'a Task>> = HashMap::new();
    for task in tasks {
        for target in task.link_targets(PARENT) {
            let Some(parent) = index.resolve(task, target) else {
                continue;
            };
            let children = out.entry(parent.file.as_path()).or_default();
            if !children.iter().any(|seen| seen.file == task.file) {
                children.push(task);
            }
        }
    }
    out
}

/// `children` as the unfinished ones, named the way a blocker is (`kind` is
/// `parent`), in the order given.
pub(crate) fn unfinished(children: &[&Task], terminal: &[String]) -> Vec<Blocker> {
    children
        .iter()
        .filter(|child| !child.is_terminal(terminal))
        .map(|child| Blocker {
            stem: child.id.clone(),
            status: child.status.as_ref().map(|status| status.name.clone()),
            kind: PARENT.to_owned(),
        })
        .collect()
}

/// `parent`'s non-terminal children within `tasks` — what a flip closing it
/// would leave behind.
#[must_use]
pub fn unfinished_children(tasks: &[Task], parent: &Task, config: &Config) -> Vec<Blocker> {
    let terminal = config.tasks.status.terminal();
    children_by_parent(tasks)
        .get(parent.file.as_path())
        .map_or_else(Vec::new, |children| unfinished(children, &terminal))
}

/// Why entering `status` is refused while `children` are unfinished:
/// `` `done` requires finished children; unfinished: a (wip) `` — shared by the
/// `mt flip` refusal and the `mt check` warning.
#[must_use]
pub fn children_violation(status: &str, children: &[Blocker]) -> String {
    format!(
        "`{status}` requires finished children; unfinished: {}",
        list_targets(children)
    )
}

/// Warn about a parent sitting in a status that requires finished children
/// ([`crate::config::Legend::requires_finished_children`]) while one is
/// unfinished. A warning, like the blocked one beside it: `mt flip --force`
/// lands exactly this state on purpose, and reopening a child puts its closed
/// parent here without either file changing.
pub(crate) fn check(tasks: &[Task], config: &Config, out: &mut Vec<Violation>) {
    let legend = &config.tasks.status;
    let by_parent = children_by_parent(tasks);
    let terminal = legend.terminal();
    for task in tasks {
        let Some(status) = task.status.as_ref() else {
            continue;
        };
        if !legend.requires_finished_children(&status.name) {
            continue;
        }
        let Some(children) = by_parent.get(task.file.as_path()) else {
            continue;
        };
        let open = unfinished(children, &terminal);
        if !open.is_empty() {
            out.push(Violation {
                task_id: task.id.clone(),
                field: "status".to_owned(),
                message: children_violation(&status.name, &open),
                location: task.field_location("status"),
                severity: Severity::Warn,
                reads: Some(crate::violation::reads_of_field("status")),
            });
        }
    }
}

#[cfg(test)]
#[path = "children_tests.rs"]
mod tests;
