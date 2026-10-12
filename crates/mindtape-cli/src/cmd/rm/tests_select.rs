//! Tests for `rm` target selection: how a `TARGET` argument list — task
//! references and filter expressions — resolves to the eligible id set. The
//! plan/apply, preflight and gitless clusters live in `cmd/rm/tests.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::HashSet;
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
            ..LoadOutcome::default()
        },
    }
}

fn readiness_of(project: &Project) -> Readiness {
    Readiness::compute(&project.outcome.tasks, &project.config)
}

/// The legend `wip`, then the terminal `done`.
const WIP_DONE: &str =
    "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// The ids `select_targets` picks for `values` among `tasks`, each an id at
/// `tasks/<id>.typ` and its status.
fn select(config: Config, tasks: &[(&str, &str)], values: &[&str]) -> HashSet<String> {
    let tasks = tasks
        .iter()
        .map(|(id, status)| task(id, &format!("tasks/{id}.typ"), Some(status)))
        .collect();
    let project = stub_project(config, tasks);
    select_targets(&project, &targets(values), &readiness_of(&project)).unwrap()
}

#[test]
fn select_targets_ref_bypasses_status() {
    // A slug reference selects exactly that task, even though it is non-terminal.
    let tasks = [("gone", "wip"), ("live", "wip")];
    assert_eq!(select(Config::default(), &tasks, &["gone"]), ids(&["gone"]));
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
    let tasks = [("a", "done"), ("b", "wip")];
    assert_eq!(
        select(parse_config(WIP_DONE).unwrap(), &tasks, &[]),
        ids(&["a"])
    );
}

#[test]
fn an_unknown_single_word_says_no_task_matches() {
    // Not a stem and no field or term: the word names a task that isn't there,
    // so the answer is about the task, not about filter syntax.
    let project = stub_project(
        Config::default(),
        vec![task("alpha", "tasks/alpha.typ", Some("wip"))],
    );
    let err = select_targets(&project, &targets(&["alpah"]), &readiness_of(&project)).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: no task: 'alpah'\nhint: did you mean alpha?\n"
    );
}

#[test]
fn a_filter_that_does_not_parse_is_reported_as_a_filter() {
    let project = stub_project(
        Config::default(),
        vec![task("a", "tasks/a.typ", Some("wip"))],
    );
    let err =
        select_targets(&project, &targets(&["status =="]), &readiness_of(&project)).unwrap_err();
    let text = crate::diag::plain(&err);
    assert!(text.starts_with("error: "), "{text}");
    assert!(!text.contains("no task"), "{text}");
}

#[test]
fn several_unresolved_targets_end_with_one_summary() {
    let project = stub_project(
        Config::default(),
        vec![task("a", "tasks/a.typ", Some("wip"))],
    );
    let err = select_targets(
        &project,
        &targets(&["a", "nope", "nada"]),
        &readiness_of(&project),
    )
    .unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: resolved 1 of 3 tasks; 2 failed\n"
    );
}

#[test]
fn a_default_filter_that_does_not_parse_names_its_key() {
    let config = parse_config("[rm]\ndefault-filter = \"(\"\n").unwrap();
    let project = stub_project(config, vec![task("a", "tasks/a.typ", Some("wip"))]);
    let err = select_targets(&project, &[], &readiness_of(&project)).unwrap_err();
    let text = crate::diag::plain(&err);
    assert!(
        text.starts_with("error: rm.default-filter: can't parse filter: '('"),
        "{text}"
    );
}

/// A task's exact stem wins over the filter word it spells: `mt rm done`
/// retires the task named `done`, not every task in the `done` status.
#[test]
fn a_stem_spelling_a_filter_word_retires_that_task() {
    let tasks = [("done", "wip"), ("shipped", "done")];
    assert_eq!(
        select(parse_config(WIP_DONE).unwrap(), &tasks, &["done"]),
        ids(&["done"])
    );
}
