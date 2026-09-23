//! Tests for `rm` target selection: how a `TARGET` argument list — task
//! references and filter expressions — resolves to the eligible id set. The
//! plan/apply, preflight and gitless clusters live in `rm_tests.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use mindtape::{Config, LoadOutcome, Readiness, Task, parse_config};

use super::select_targets;
use super::tests::{ids, task};
use crate::util::Project;

/// Build a `select_targets` argument list from string slices.
fn targets(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

/// An in-memory project — no disk needed for slug/filter selection logic.
fn stub_project(config: Config, tasks: Vec<Task>) -> Project {
    Project {
        root: PathBuf::from("."),
        config,
        outcome: LoadOutcome {
            tasks,
            errors: vec![],
        },
    }
}

fn readiness_of(project: &Project) -> Readiness {
    Readiness::compute(&project.outcome.tasks, &project.config)
}

#[test]
fn select_targets_ref_bypasses_status() {
    // A slug reference selects exactly that task, even though it is non-terminal.
    let project = stub_project(
        Config::default(),
        vec![
            task("gone", "tasks/gone.typ", Some("wip")),
            task("live", "tasks/live.typ", Some("wip")),
        ],
    );
    let got = select_targets(&project, &targets(&["gone"]), &readiness_of(&project)).unwrap();
    assert_eq!(got, ids(&["gone"]));
}

#[test]
fn select_targets_filter_selects_every_match() {
    let project = stub_project(
        Config::default(),
        vec![
            task("a", "tasks/a.typ", Some("done")),
            task("b", "tasks/b.typ", Some("wip")),
            task("c", "tasks/c.typ", Some("done")),
        ],
    );
    let got = select_targets(
        &project,
        &targets(&["status == \"done\""]),
        &readiness_of(&project),
    )
    .unwrap();
    assert_eq!(got, ids(&["a", "c"]));
}

#[test]
fn select_targets_two_refs_union() {
    // Two refs in one invocation retire both — the whole point of making
    // TARGET variadic.
    let project = stub_project(
        Config::default(),
        vec![
            task("a", "tasks/a.typ", Some("wip")),
            task("b", "tasks/b.typ", Some("wip")),
            task("c", "tasks/c.typ", Some("wip")),
        ],
    );
    let got = select_targets(&project, &targets(&["a", "b"]), &readiness_of(&project)).unwrap();
    assert_eq!(got, ids(&["a", "b"]));
}

#[test]
fn select_targets_ref_and_filter_union() {
    // A ref and a filter together retire the union of what each selects.
    let project = stub_project(
        Config::default(),
        vec![
            task("a", "tasks/a.typ", Some("wip")),
            task("b", "tasks/b.typ", Some("done")),
            task("c", "tasks/c.typ", Some("wip")),
        ],
    );
    let got = select_targets(
        &project,
        &targets(&["a", "status == \"done\""]),
        &readiness_of(&project),
    )
    .unwrap();
    assert_eq!(got, ids(&["a", "b"]));
}

#[test]
fn select_targets_same_task_named_twice_retires_once() {
    let project = stub_project(
        Config::default(),
        vec![task("a", "tasks/a.typ", Some("wip"))],
    );
    let got = select_targets(&project, &targets(&["a", "a"]), &readiness_of(&project)).unwrap();
    assert_eq!(got, ids(&["a"]));
}

#[test]
fn select_targets_default_filter_is_terminal() {
    // No target -> the built-in `terminal` default filter sweeps terminal tasks.
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap();
    let project = stub_project(
        config,
        vec![
            task("a", "tasks/a.typ", Some("done")),
            task("b", "tasks/b.typ", Some("wip")),
        ],
    );
    let got = select_targets(&project, &[], &readiness_of(&project)).unwrap();
    assert_eq!(got, ids(&["a"]));
}

#[test]
fn select_targets_unresolvable_token_errors() {
    // Not a slug and not a valid boolean filter -> error, not a silent empty set.
    let project = stub_project(
        Config::default(),
        vec![task("a", "tasks/a.typ", Some("wip"))],
    );
    assert!(select_targets(&project, &targets(&["nope"]), &readiness_of(&project)).is_err());
}
