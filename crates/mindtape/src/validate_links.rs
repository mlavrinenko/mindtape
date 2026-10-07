//! Validation for the typed `links:` edge field, split out of `validate.rs`
//! (file-size gate): target resolution, per-kind cardinality, required-note
//! enforcement, unknown-kind rejection, and cycle detection over every
//! acyclic-flagged kind (`depends-on` and `parent` ship flagged this way; see
//! [`crate::link::acyclic_kinds`]).
//!
//! A kind whose `[[tasks.links.kinds]] validate` is not `local-task-path`
//! points outside the backlog (a commit, a pull request, a build). Its targets
//! are checked for *shape* by [`crate::config::links_validate`] instead of being
//! resolved against the task set, and they take no part in cardinality or
//! cycles — `acyclic_kinds` already drops them. The note rule still applies, so
//! such a kind can demand a reason like any other.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::Config;
use crate::config::links_validate::Validators;
use crate::graph::{DepIndex, toposort_kind};
use crate::link::{
    Link, LinksConfig, acyclic_kinds, blocking_kinds, kind_targets_tasks, resolve_kind,
};
use crate::model::{Task, resolve_link_path};
use crate::ready::{collect_blockers, unblocked_violation};
use crate::severity::Severity;
use crate::validate::Violation;
use crate::violation::ViolationKind;

/// Validate every task's `links:` edges against `config`, appending violations
/// to `out`: resolution (or target-shape validation) and per-kind
/// cardinality/note rules run per task; cycle detection runs once over the
/// whole set, per acyclic-flagged kind.
pub(crate) fn check_all(tasks: &[Task], config: &Config, out: &mut Vec<Violation>) {
    let files: HashSet<&Path> = tasks.iter().map(|task| task.file.as_path()).collect();
    let validators = Validators::build(&config.tasks.links);
    for task in tasks {
        check_task(task, &files, config, &validators, out);
    }
    for kind in acyclic_kinds(&config.tasks.links) {
        check_cycle(tasks, kind, out);
    }
    check_unblocked(tasks, config, out);
    crate::children::check(tasks, config, out);
}

/// Warn about a task sitting in an `unblocked = true` status while it waits on
/// an unfinished blocking-kind target. A warning, not an error: `mt flip
/// --force` lands exactly this state on purpose, and reopening a dependency
/// puts every finished dependant here without either file changing.
fn check_unblocked(tasks: &[Task], config: &Config, out: &mut Vec<Violation>) {
    let legend = &config.tasks.status;
    if !legend.entries.iter().any(|entry| entry.unblocked) {
        return;
    }
    let index = DepIndex::build(tasks);
    let kinds = blocking_kinds(&config.tasks.links);
    let terminal = legend.terminal();
    for task in tasks {
        let Some(status) = task.status.as_ref() else {
            continue;
        };
        if !legend.requires_unblocked(&status.name) {
            continue;
        }
        let blockers = collect_blockers(task, &index, &kinds, &terminal);
        if !blockers.is_empty() {
            out.push(Violation {
                task_id: task.id.clone(),
                field: "status".to_owned(),
                message: unblocked_violation(&status.name, &blockers),
                location: task.field_location("status"),
                severity: Severity::Warn,
                kind: ViolationKind::Other,
                reads: Some(crate::violation::reads_of_field("status")),
            });
        }
    }
}

/// Validate one task's edges: resolution against `files` (or target-shape
/// validation, for a kind that points outside the backlog), unknown-kind
/// rejection, the note rule, and (once all edges are seen) cardinality.
fn check_task(
    task: &Task,
    files: &HashSet<&Path>,
    config: &Config,
    validators: &Validators,
    out: &mut Vec<Violation>,
) {
    let links = &config.tasks.links;
    let mut per_kind: HashMap<&str, u32> = HashMap::new();
    for link in &task.links {
        let to_task = kind_targets_tasks(&link.kind, links);
        if to_task {
            check_resolution(task, link, files, out);
        } else if let Some(message) = validators.problem(&link.kind, &link.task) {
            push(out, task, kind_field(link), message);
        }
        match resolve_kind(&link.kind, links) {
            None => push(out, task, kind_field(link), unknown_kind_message(link)),
            Some(rule) => {
                if to_task {
                    *per_kind.entry(link.kind.as_str()).or_insert(0) += 1;
                }
                check_note_rule(task, link, rule.note, out);
            }
        }
    }
    check_cardinality(task, &per_kind, links, out);
}

/// Flag every kind whose edge count on `task` exceeds its cardinality cap.
/// Only kinds that target tasks reach here — a cap counts task-graph edges.
fn check_cardinality(
    task: &Task,
    per_kind: &HashMap<&str, u32>,
    links: &LinksConfig,
    out: &mut Vec<Violation>,
) {
    for (kind, count) in per_kind {
        if let Some(max) = resolve_kind(kind, links).and_then(|rule| rule.max)
            && *count > max
        {
            push(
                out,
                task,
                (*kind).to_owned(),
                format!("`{kind}` allows at most {max} edge(s), found {count}"),
            );
        }
    }
}

/// The violation field for an edge: the kind name when known, else the
/// generic `links` (a bare string in the list parses to an empty kind).
fn kind_field(link: &Link) -> String {
    if link.kind.is_empty() {
        "links".to_owned()
    } else {
        link.kind.clone()
    }
}

fn unknown_kind_message(link: &Link) -> String {
    if link.kind.is_empty() {
        format!(
            "link target `{}` has no kind; use a helper like depends-on(...), related(...), or edge(kind, ...)",
            link.task
        )
    } else {
        format!("`{}` is not a configured link kind", link.kind)
    }
}

fn check_resolution(task: &Task, link: &Link, files: &HashSet<&Path>, out: &mut Vec<Violation>) {
    let resolved = resolve_link_path(&task.file, &link.task);
    let found = resolved.as_deref().is_some_and(|p| files.contains(p));
    if !found {
        let display = resolved
            .as_deref()
            .unwrap_or(Path::new(&link.task))
            .display()
            .to_string();
        let mut breach = breach(task, kind_field(link), unresolved_message(&display));
        breach.kind = ViolationKind::UnresolvedLink;
        out.push(breach);
    }
}

/// What an edge to a task that is not in the set says.
fn unresolved_message(display: &str) -> String {
    format!("link target `{display}` not found")
}

/// Whether `violation` is an edge's target missing from the set, which only the
/// set around the task can settle.
pub(crate) fn is_unresolved(violation: &Violation) -> bool {
    violation.kind == ViolationKind::UnresolvedLink
}

fn check_note_rule(
    task: &Task,
    link: &Link,
    note_rule: crate::link::NoteRule,
    out: &mut Vec<Violation>,
) {
    use crate::link::NoteRule;
    match note_rule {
        NoteRule::Required if link.note.as_deref().unwrap_or("").trim().is_empty() => push(
            out,
            task,
            link.kind.clone(),
            format!("`{}` edge to `{}` requires a note", link.kind, link.task),
        ),
        NoteRule::Forbidden if link.note.is_some() => push(
            out,
            task,
            link.kind.clone(),
            format!(
                "`{}` edge to `{}` must not carry a note",
                link.kind, link.task
            ),
        ),
        NoteRule::Required | NoteRule::Optional | NoteRule::Forbidden => {}
    }
}

/// Flag a cycle over one kind's edges: no valid order exists, so the toposort
/// fails loud. Attributed to the first task on the loop; the message names
/// the full chain.
fn check_cycle(tasks: &[Task], kind: &str, out: &mut Vec<Violation>) {
    if let Err(cycle) = toposort_kind(tasks, kind) {
        let task_id = cycle.members.first().cloned().unwrap_or_default();
        // A cycle's members are tasks of `tasks`, so the member is always found;
        // there is no path to invent when it is not.
        let Some(member) = tasks.iter().find(|task| task.id == task_id) else {
            return;
        };
        let location = member.field_location("links");
        out.push(Violation {
            task_id,
            field: kind.to_owned(),
            message: format!("{kind} cycle: {cycle}"),
            location,
            severity: Severity::Error,
            kind: ViolationKind::Other,
            reads: Some(crate::violation::reads_of_field("links")),
        });
    }
}

fn push(out: &mut Vec<Violation>, task: &Task, field: String, message: String) {
    out.push(breach(task, field, message));
}

/// An error about the task's `links` field.
fn breach(task: &Task, field: String, message: String) -> Violation {
    Violation {
        task_id: task.id.clone(),
        location: task.field_location("links"),
        field,
        message,
        severity: Severity::Error,
        kind: ViolationKind::Other,
        reads: Some(crate::violation::reads_of_field("links")),
    }
}

#[cfg(test)]
#[path = "validate_links_tests.rs"]
mod tests;
