#![allow(clippy::unwrap_used)]

use super::{Cycle, edges, find_referrer, resolve_edge, toposort, toposort_kind};
use crate::model::Task;
use crate::test_support::{bare_task, link_edge};

fn task(id: &str, deps: &[&str]) -> Task {
    task_kind(id, "depends-on", deps)
}

fn task_kind(id: &str, kind: &str, deps: &[&str]) -> Task {
    let mut task = bare_task(id);
    task.links = deps
        .iter()
        .map(|dep| link_edge(&format!("{dep}.typ"), kind))
        .collect();
    task
}

/// Position of `id` in the toposorted order.
fn pos(order: &[&Task], id: &str) -> usize {
    order
        .iter()
        .position(|t| t.id == id)
        .expect("task in order")
}

#[test]
fn empty_graph_sorts_to_empty() {
    assert!(toposort(&[]).unwrap().is_empty());
}

#[test]
fn independent_tasks_keep_input_order() {
    let tasks = [task("a", &[]), task("b", &[]), task("c", &[])];
    let order = toposort(&tasks).unwrap();
    let ids: Vec<&str> = order.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["a", "b", "c"]);
}

#[test]
fn dependency_precedes_dependent() {
    // a -> b -> c, so the order must be c, b, a.
    let tasks = [task("a", &["b"]), task("b", &["c"]), task("c", &[])];
    let order = toposort(&tasks).unwrap();
    assert!(pos(&order, "c") < pos(&order, "b"));
    assert!(pos(&order, "b") < pos(&order, "a"));
}

#[test]
fn diamond_joins_after_both_branches() {
    // a depends on b and c; b and c both depend on d.
    let tasks = [
        task("a", &["b", "c"]),
        task("b", &["d"]),
        task("c", &["d"]),
        task("d", &[]),
    ];
    let order = toposort(&tasks).unwrap();
    assert!(pos(&order, "d") < pos(&order, "b"));
    assert!(pos(&order, "d") < pos(&order, "c"));
    assert!(pos(&order, "b") < pos(&order, "a"));
    assert!(pos(&order, "c") < pos(&order, "a"));
}

#[test]
fn dangling_dependency_is_not_an_edge() {
    // Target names no loaded task: ignored, no cycle, task still ordered.
    let tasks = [task("a", &["missing"])];
    let order = toposort(&tasks).unwrap();
    assert_eq!(order.len(), 1);
}

#[test]
fn detects_two_node_cycle() {
    let tasks = [task("a", &["b"]), task("b", &["a"])];
    let cycle = toposort(&tasks).unwrap_err();
    assert!(cycle.members.contains(&"a".to_owned()));
    assert!(cycle.members.contains(&"b".to_owned()));
}

#[test]
fn detects_self_cycle() {
    let cycle = toposort(&[task("a", &["a"])]).unwrap_err();
    assert_eq!(cycle.members, ["a"]);
}

#[test]
fn detects_longer_cycle() {
    let tasks = [task("a", &["b"]), task("b", &["c"]), task("c", &["a"])];
    let cycle = toposort(&tasks).unwrap_err();
    assert_eq!(cycle.members.len(), 3);
}

#[test]
fn cycle_display_closes_loop() {
    let cycle = Cycle {
        members: vec!["a".to_owned(), "b".to_owned()],
    };
    assert_eq!(cycle.to_string(), "a \u{2192} b \u{2192} a");
}

#[test]
fn toposort_kind_walks_only_the_named_kind() {
    // `a` has both a `depends-on` and a `parent` edge to `b`; toposort_kind
    // over "parent" must ignore the depends-on edge entirely.
    let mut a = bare_task("a");
    a.links = vec![
        link_edge("b.typ", "depends-on"),
        link_edge("b.typ", "parent"),
    ];
    let b = bare_task("b");
    let tasks = [a, b];
    let order = toposort_kind(&tasks, "parent").unwrap();
    assert_eq!(order.len(), 2);
    assert!(pos(&order, "b") < pos(&order, "a"));
}

#[test]
fn toposort_kind_detects_parent_cycle_independent_of_depends_on() {
    // a -> b -> a via `parent`, while `depends-on` is acyclic (a -> b only):
    // the parent cycle must surface even though depends-on does not.
    let mut a = bare_task("a");
    a.links = vec![
        link_edge("b.typ", "depends-on"),
        link_edge("b.typ", "parent"),
    ];
    let mut b = bare_task("b");
    b.links = vec![link_edge("a.typ", "parent")];

    assert!(toposort_kind(&[a.clone(), b.clone()], "depends-on").is_ok());
    let cycle = toposort_kind(&[a, b], "parent").unwrap_err();
    assert!(cycle.members.contains(&"a".to_owned()));
    assert!(cycle.members.contains(&"b".to_owned()));
}

#[test]
fn resolve_edge_finds_the_forward_target() {
    let tasks = [
        task_kind("spawn-2", "recurred-from", &["spawn-1"]),
        bare_task("spawn-1"),
    ];
    let resolved = resolve_edge(&tasks, &tasks[0], "recurred-from").unwrap();
    assert_eq!(resolved.id, "spawn-1");
}

#[test]
fn resolve_edge_none_when_no_edge_of_kind() {
    let tasks = [bare_task("a"), bare_task("b")];
    assert!(resolve_edge(&tasks, &tasks[0], "recurred-from").is_none());
}

#[test]
fn find_referrer_finds_the_successor() {
    let tasks = [
        bare_task("original"),
        task_kind("spawn-1", "recurred-from", &["original"]),
    ];
    let successor = find_referrer(&tasks, &tasks[0], "recurred-from").unwrap();
    assert_eq!(successor.id, "spawn-1");
}

#[test]
fn find_referrer_none_without_a_successor() {
    let tasks = [bare_task("original")];
    assert!(find_referrer(&tasks, &tasks[0], "recurred-from").is_none());
}

#[test]
fn edges_resolves_every_kind_in_authored_order() {
    let mut a = bare_task("a");
    a.links = vec![
        link_edge("b.typ", "depends-on"),
        link_edge("c.typ", "parent"),
    ];
    let tasks = [a, bare_task("b"), bare_task("c")];
    let edges = edges(&tasks);
    let seen: Vec<(&str, &str, &str)> = edges
        .iter()
        .map(|e| (e.from.id.as_str(), e.to.id.as_str(), e.kind))
        .collect();
    assert_eq!(seen, [("a", "b", "depends-on"), ("a", "c", "parent")]);
}

#[test]
fn edges_drops_targets_outside_the_set() {
    // `b` is not in the passed slice, so a's edge to it is not induced.
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "depends-on")];
    let tasks = [a];
    assert!(edges(&tasks).is_empty());
}
