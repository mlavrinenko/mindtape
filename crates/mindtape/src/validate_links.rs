//! Validation for the typed `links:` edge field, split out of `validate.rs`
//! (file-size gate): target resolution, per-kind cardinality, required-note
//! enforcement, unknown-kind rejection, and cycle detection over every
//! acyclic-flagged kind (`depends-on` and `parent` ship flagged this way; see
//! [`crate::link::acyclic_kinds`]).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::Config;
use crate::graph::toposort_kind;
use crate::link::{Link, acyclic_kinds, resolve_kind};
use crate::model::{Task, resolve_link_path};
use crate::validate::Violation;

/// Validate every task's `links:` edges against `config`, appending violations
/// to `out`: resolution and per-kind cardinality/note rules run per task;
/// cycle detection runs once over the whole set, per acyclic-flagged kind.
pub(crate) fn check_all(tasks: &[Task], config: &Config, out: &mut Vec<Violation>) {
    let files: HashSet<&Path> = tasks.iter().map(|task| task.file.as_path()).collect();
    for task in tasks {
        check_task(task, &files, config, out);
    }
    for kind in acyclic_kinds(&config.tasks.links) {
        check_cycle(tasks, kind, out);
    }
}

/// Validate one task's edges: resolution against `files`, unknown-kind
/// rejection, the note rule, and (once all edges are seen) cardinality.
fn check_task(task: &Task, files: &HashSet<&Path>, config: &Config, out: &mut Vec<Violation>) {
    let mut per_kind: HashMap<&str, u32> = HashMap::new();
    for link in &task.links {
        check_resolution(task, link, files, out);
        match resolve_kind(&link.kind, &config.tasks.links) {
            None => push(out, task, kind_field(link), unknown_kind_message(link)),
            Some(rule) => {
                *per_kind.entry(link.kind.as_str()).or_insert(0) += 1;
                check_note_rule(task, link, rule.note, out);
            }
        }
    }
    for (kind, count) in per_kind {
        if let Some(max) = resolve_kind(kind, &config.tasks.links).and_then(|rule| rule.max)
            && count > max
        {
            push(
                out,
                task,
                kind.to_owned(),
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
        push(
            out,
            task,
            kind_field(link),
            format!("link target `{display}` not found"),
        );
    }
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
        out.push(Violation {
            task_id: cycle.members.first().cloned().unwrap_or_default(),
            field: kind.to_owned(),
            message: format!("{kind} cycle: {cycle}"),
            location: None,
        });
    }
}

fn push(out: &mut Vec<Violation>, task: &Task, field: String, message: String) {
    out.push(Violation {
        task_id: task.id.clone(),
        field,
        message,
        location: None,
    });
}

#[cfg(test)]
#[path = "validate_links_tests.rs"]
mod tests;
