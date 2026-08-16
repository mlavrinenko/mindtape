#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{DEFAULT_DRIFT_ADVICE, DriftConfig, EstimateConfig};
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::test_support::bare_task;
use crate::validate::validate;

/// A legend plus a `files` estimate bounded to 1..5 — the shipped shape of a
/// file budget, and the fixture every bound test below measures against.
fn config() -> Config {
    parse_config(
        r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.estimate]
units = ["files"]

[tasks.estimate.max]
files = 5

[tasks.estimate.min]
files = 1
"#,
    )
    .unwrap()
}

/// The `[tasks.estimate]` section of [`config`], on its own.
fn estimate() -> EstimateConfig {
    config().tasks.estimate
}

/// An estimate dict declaring `files`.
fn files(count: f64) -> BTreeMap<String, f64> {
    BTreeMap::from([("files".to_owned(), count)])
}

/// A task in `status` declaring `count` files.
fn task(id: &str, status: &str, count: f64) -> Task {
    Task {
        status: Some(Status {
            name: status.to_owned(),
            date: None,
            note: None,
        }),
        estimate: files(count),
        ..bare_task(id)
    }
}

#[test]
fn an_estimate_under_the_cap_is_clean() {
    assert!(estimate().bound_problems(&files(4.0)).is_empty());
}

#[test]
fn an_estimate_at_the_cap_is_clean() {
    assert!(
        estimate().bound_problems(&files(5.0)).is_empty(),
        "`max` is inclusive: the declared cap is a legal estimate"
    );
}

#[test]
fn an_estimate_over_the_cap_is_a_problem_naming_the_split() {
    let problems = estimate().bound_problems(&files(6.0));
    assert_eq!(problems.len(), 1, "{problems:?}");
    let message = problems.first().unwrap();
    assert!(
        message.contains("6 files is over the 5 allowed"),
        "{message}"
    );
    assert!(
        message.contains("split the task before starting"),
        "the message must name the action, not just the breach: {message}"
    );
}

#[test]
fn an_estimate_below_the_floor_is_a_problem() {
    let problems = estimate().bound_problems(&files(0.0));
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems.first().unwrap().contains("under the 1 required"),
        "{problems:?}"
    );
}

#[test]
fn an_estimate_at_the_floor_is_clean() {
    assert!(estimate().bound_problems(&files(1.0)).is_empty());
}

#[test]
fn a_unit_with_no_declared_bound_is_unbounded() {
    let estimate = BTreeMap::from([("hours".to_owned(), 900.0)]);
    assert!(
        super::EstimateConfig::default()
            .bound_problems(&estimate)
            .is_empty()
    );
    assert!(config().tasks.estimate.bound_problems(&estimate).is_empty());
}

#[test]
fn an_over_cap_estimate_fails_validation() {
    let violations = validate(&[task("big", "wip", 9.0)], &config());
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations.first().unwrap().field, "estimate");
}

#[test]
fn a_terminal_task_is_exempt_from_the_bounds() {
    let violations = validate(&[task("shipped", "done", 9.0)], &config());
    assert!(
        violations.is_empty(),
        "a finished task cannot be split; its estimate must not fail the backlog: {violations:?}"
    );
}

#[test]
fn the_unit_allow_list_still_applies() {
    let mut over = task("wrong-unit", "wip", 3.0);
    over.estimate = BTreeMap::from([("furlongs".to_owned(), 3.0)]);
    let violations = validate(&[over], &config());
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations
            .first()
            .unwrap()
            .message
            .contains("not an allowed estimate unit"),
        "{violations:?}"
    );
}

#[test]
fn drift_defaults_target_a_file_budget_on_the_working_state() {
    let drift = DriftConfig::default();
    assert_eq!(drift.unit, "files");
    assert_eq!(drift.in_flight, "status == \"wip\"");
    assert_eq!(drift.line_multiple, 200);
    assert!(drift.exclude.is_empty(), "nothing is excluded by default");
    assert_eq!(drift.advice(), DEFAULT_DRIFT_ADVICE);
    assert!(
        drift.advice().contains("split the remaining work"),
        "the default advice must name the next move: {}",
        drift.advice()
    );
}

#[test]
fn a_configured_message_replaces_the_default_advice() {
    let drift = DriftConfig {
        message: Some("stop and rescope".to_owned()),
        ..DriftConfig::default()
    };
    assert_eq!(drift.advice(), "stop and rescope");
}

#[test]
fn an_exclude_list_parses_off_tasks_estimate_drift() {
    let toml = r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.estimate]
units = ["files"]

[tasks.estimate.drift]
exclude = ["docs/generated/**", "*.lock"]
"#;
    let drift = parse_config(toml).unwrap().tasks.estimate.drift;
    assert_eq!(
        drift.exclude,
        vec!["docs/generated/**".to_owned(), "*.lock".to_owned()]
    );
}
