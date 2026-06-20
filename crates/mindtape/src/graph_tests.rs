#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{Cycle, toposort};
use crate::model::Task;

fn task(id: &str, deps: &[&str]) -> Task {
    Task {
        id: id.to_owned(),
        file: format!("tasks/{id}.typ").into(),
        title: id.to_owned(),
        status: None,
        priority: None,
        difficulty: None,
        kind: None,
        estimate: BTreeMap::new(),
        depends_on: deps.iter().map(|dep| format!("{dep}.typ")).collect(),
        relates_to: Vec::new(),
        supersedes: Vec::new(),
        duplicates: Vec::new(),
        tags: Vec::new(),
        due: None,
        start: None,
        reviewed: None,
        extras: BTreeMap::new(),
    }
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
