//! `children`/`progress` over `parent` edges, and the unfinished-children
//! listing the `mt flip` refusal and the `mt check` warning share.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{children_violation, unfinished_children};
use crate::config::Config;
use crate::model::Task;
use crate::test_support::{dep_config, dep_task, link_edge};

fn config() -> Config {
    dep_config("")
}

/// A task in `status` whose only edge is `parent` to `parent`.
fn child(id: &str, status: &str, parent: &str) -> Task {
    let mut task = dep_task(id, status, &[]);
    task.links = vec![link_edge(parent, "parent")];
    task
}

fn annotated(mut tasks: Vec<Task>) -> Vec<Task> {
    crate::dep_rank::annotate(&mut tasks, &config());
    tasks
}

fn find<'a>(tasks: &'a [Task], id: &str) -> &'a Task {
    tasks.iter().find(|task| task.id == id).unwrap()
}

#[test]
fn a_parent_counts_its_children_and_the_finished_ones() {
    let tasks = annotated(vec![
        dep_task("epic", "wip", &[]),
        child("a", "done", "epic.typ"),
        child("b", "wip", "epic.typ"),
        child("c", "wip", "epic.typ"),
    ]);
    let epic = find(&tasks, "epic");
    assert_eq!((epic.children, epic.children_done), (3, 1));
    assert_eq!(epic.progress(), Some(1.0 / 3.0));
}

#[test]
fn a_leaf_has_no_children_and_no_progress() {
    let tasks = annotated(vec![
        dep_task("leaf", "wip", &[]),
        child("a", "wip", "leaf.typ"),
    ]);
    let leaf = find(&tasks, "a");
    assert_eq!((leaf.children, leaf.children_done), (0, 0));
    assert_eq!(leaf.progress(), None);
}

#[test]
fn every_child_finished_is_progress_one() {
    let tasks = annotated(vec![
        dep_task("epic", "wip", &[]),
        child("a", "done", "epic.typ"),
    ]);
    assert_eq!(find(&tasks, "epic").progress(), Some(1.0));
}

#[test]
fn only_parent_edges_count_and_a_dangling_one_counts_nowhere() {
    let tasks = annotated(vec![
        dep_task("epic", "wip", &[]),
        dep_task("dependant", "wip", &["epic.typ"]),
        child("orphan", "wip", "missing.typ"),
    ]);
    assert_eq!(find(&tasks, "epic").children, 0);
}

#[test]
fn a_duplicated_parent_edge_counts_the_child_once() {
    let mut twice = child("a", "wip", "epic.typ");
    twice.links.push(link_edge("epic.typ", "parent"));
    let tasks = annotated(vec![dep_task("epic", "wip", &[]), twice]);
    assert_eq!(find(&tasks, "epic").children, 1);
}

#[test]
fn unfinished_children_name_each_non_terminal_child() {
    let tasks = vec![
        dep_task("epic", "wip", &[]),
        child("a", "done", "epic.typ"),
        child("b", "wip", "epic.typ"),
    ];
    let open = unfinished_children(&tasks, &tasks[0], &config());
    let named: Vec<(&str, Option<&str>, &str)> = open
        .iter()
        .map(|c| (c.stem.as_str(), c.status.as_deref(), c.kind.as_str()))
        .collect();
    assert_eq!(named, [("b", Some("wip"), "parent")]);
    assert_eq!(
        children_violation("done", &open),
        "`done` requires finished children; unfinished: b (wip)"
    );
}

#[test]
fn check_warns_about_a_closed_parent_over_an_unfinished_child() {
    // `unblocked = true` lands on `done`, the last entry dep_config writes.
    let closing = dep_config("unblocked = true\n");
    let tasks = vec![
        dep_task("closed", "done", &[]),
        child("open", "wip", "closed.typ"),
        dep_task("finished", "done", &[]),
        child("closed-child", "done", "finished.typ"),
        dep_task("working", "wip", &[]),
        child("open-too", "wip", "working.typ"),
    ];
    let mut violations = Vec::new();
    super::check(&tasks, &closing, &mut violations);
    assert_eq!(
        crate::test_support::violation_triples(&violations),
        [(
            "closed",
            "status",
            "`done` requires finished children; unfinished: open (wip)"
        )]
    );
    assert_eq!(
        violations.first().and_then(|v| v.severity),
        Some(crate::Severity::Warn)
    );
    let mut none = Vec::new();
    super::check(&tasks, &config(), &mut none);
    assert!(none.is_empty(), "no unblocked status, no warning");
}
