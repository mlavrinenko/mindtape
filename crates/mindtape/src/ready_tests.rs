#![allow(clippy::unwrap_used)]

use super::{Readiness, ReadyState};
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::test_support::bare_task;

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
    task.depends_on = deps.iter().map(|d| (*d).to_owned()).collect();
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
