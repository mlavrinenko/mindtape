#![allow(clippy::unwrap_used)]

use super::{Readiness, ReadyState};
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::test_support::{bare_task, dep_edge};

fn config() -> Config {
    parse_config(
        r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true
"#,
    )
    .unwrap()
}

fn task(id: &str, status: &str, deps: &[&str]) -> Task {
    let mut task = bare_task(id);
    task.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    task.links = deps.iter().map(|d| dep_edge(d)).collect();
    task
}

#[test]
fn no_deps_is_ready() {
    let tasks = [task("a", "wip", &[])];
    let ready = Readiness::compute(&tasks, &config());
    assert_eq!(ready.state(&tasks[0]), ReadyState::Ready);
    assert!(ready.is_ready(&tasks[0]));
}

#[test]
fn all_terminal_deps_is_ready() {
    let tasks = [task("dep", "done", &[]), task("a", "wip", &["dep.typ"])];
    let ready = Readiness::compute(&tasks, &config());
    assert_eq!(ready.state(&tasks[1]), ReadyState::Ready);
}

#[test]
fn partial_terminal_deps_is_blocked() {
    let tasks = [
        task("done-dep", "done", &[]),
        task("open-dep", "wip", &[]),
        task("a", "wip", &["done-dep.typ", "open-dep.typ"]),
    ];
    let ready = Readiness::compute(&tasks, &config());
    assert_eq!(ready.state(&tasks[2]), ReadyState::Blocked);
    assert!(ready.is_blocked(&tasks[2]));
}

#[test]
fn terminal_self_is_settled() {
    let tasks = [task("a", "done", &[])];
    let ready = Readiness::compute(&tasks, &config());
    assert_eq!(ready.state(&tasks[0]), ReadyState::Settled);
    assert!(!ready.is_ready(&tasks[0]));
    assert!(!ready.is_blocked(&tasks[0]));
}

#[test]
fn dangling_dep_is_blocked() {
    // Names no loaded task: cannot confirm terminal, so conservatively blocked.
    let tasks = [task("a", "wip", &["missing.typ"])];
    let ready = Readiness::compute(&tasks, &config());
    assert_eq!(ready.state(&tasks[0]), ReadyState::Blocked);
}

#[test]
fn empty_readiness_settles_everything() {
    let tasks = [task("a", "wip", &[])];
    let ready = Readiness::empty();
    assert_eq!(ready.state(&tasks[0]), ReadyState::Settled);
}

#[test]
fn related_edges_never_gate_readiness() {
    // `related` is not flagged blocking, unlike `depends-on`: an unresolved
    // `related` target must not block a task that has no open dependency.
    use crate::test_support::link_edge;

    let mut a = bare_task("a");
    a.status = Some(Status {
        name: "wip".to_owned(),
        date: None,
        note: None,
    });
    a.links = vec![link_edge("missing.typ", "related")];
    let tasks = [a];
    let ready = Readiness::compute(&tasks, &config());
    assert_eq!(ready.state(&tasks[0]), ReadyState::Ready);
}

#[test]
fn custom_blocking_kind_gates_readiness() {
    // A project can opt a custom kind into gating readiness via
    // `[[tasks.links.kinds]] blocking = true`; a task with an unresolved
    // custom-kind edge is blocked, just like an unresolved `depends-on`.
    use crate::link::{LinkKind, NoteRule};
    use crate::test_support::link_edge;

    let mut cfg = config();
    cfg.tasks.links.kinds = vec![LinkKind {
        name: "needs".to_owned(),
        max: None,
        note: NoteRule::Optional,
        blocking: true,
        acyclic: false,
    }];

    let mut a = bare_task("a");
    a.status = Some(Status {
        name: "wip".to_owned(),
        date: None,
        note: None,
    });
    a.links = vec![link_edge("missing.typ", "needs")];
    let tasks = [a];
    let ready = Readiness::compute(&tasks, &cfg);
    assert_eq!(ready.state(&tasks[0]), ReadyState::Blocked);
}
