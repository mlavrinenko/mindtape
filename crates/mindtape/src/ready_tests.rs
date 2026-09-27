#![allow(clippy::unwrap_used)]

use super::{Readiness, ReadyState};
use crate::config::Config;
use crate::model::Status;
use crate::test_support::{bare_task, dep_config, dep_task as task};

fn config() -> Config {
    dep_config("")
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
    use crate::link::LinkKind;
    use crate::test_support::link_edge;

    let mut cfg = config();
    cfg.tasks.links.kinds = vec![LinkKind {
        name: "needs".to_owned(),
        blocking: true,
        ..LinkKind::default()
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

#[test]
fn blockers_name_each_unfinished_target_with_its_status() {
    let tasks = [
        task("done-dep", "done", &[]),
        task("open-dep", "wip", &[]),
        task("a", "wip", &["done-dep.typ", "open-dep.typ", "missing.typ"]),
    ];
    let ready = Readiness::compute(&tasks, &config());
    let blockers: Vec<(&str, Option<&str>, &str)> = ready
        .blockers(&tasks[2])
        .iter()
        .map(|b| (b.stem.as_str(), b.status.as_deref(), b.kind.as_str()))
        .collect();
    assert_eq!(
        blockers,
        [
            ("open-dep", Some("wip"), "depends-on"),
            ("missing", None, "depends-on"),
        ]
    );
}

#[test]
fn a_ready_or_settled_task_has_no_blockers() {
    let tasks = [task("dep", "done", &[]), task("a", "wip", &["dep.typ"])];
    let ready = Readiness::compute(&tasks, &config());
    assert!(ready.blockers(&tasks[0]).is_empty());
    assert!(ready.blockers(&tasks[1]).is_empty());
}
