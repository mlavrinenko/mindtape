#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{DEFAULT_DRIFT_ADVICE, DriftConfig, EstimateBounds};
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

/// Every bound breach `task` commits under `config`.
fn problems(config: &Config, task: &Task) -> Vec<String> {
    EstimateBounds::compile(&config.tasks.estimate).problems(task, config)
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
    assert!(problems(&config(), &task("t", "wip", 4.0)).is_empty());
}

#[test]
fn an_estimate_at_the_cap_is_clean() {
    assert!(
        problems(&config(), &task("t", "wip", 5.0)).is_empty(),
        "`max` is inclusive: the declared cap is a legal estimate"
    );
}

#[test]
fn an_estimate_over_the_cap_is_a_problem_naming_the_split() {
    let problems = problems(&config(), &task("t", "wip", 6.0));
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
    let problems = problems(&config(), &task("t", "wip", 0.0));
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems.first().unwrap().contains("under the 1 required"),
        "{problems:?}"
    );
}

#[test]
fn an_estimate_at_the_floor_is_clean() {
    assert!(problems(&config(), &task("t", "wip", 1.0)).is_empty());
}

#[test]
fn a_unit_with_no_declared_bound_is_unbounded() {
    let mut hours = task("t", "wip", 1.0);
    hours.estimate = BTreeMap::from([("hours".to_owned(), 900.0)]);
    assert!(problems(&Config::default(), &hours).is_empty());
    assert!(problems(&config(), &hours).is_empty());
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

/// The dogfood shape: judgment work (`difficulty >= 3`) capped at 5 files with
/// a reason, everything else at 15, and an `hours` ceiling only the fallback
/// declares.
const GATED: &str = r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.estimate]
units = ["files", "hours"]

[[tasks.estimate.max]]
when = "difficulty >= 3"
files = 5
message = "judgment work stays small"

[[tasks.estimate.max]]
files = 15
hours = 40

[[tasks.estimate.min]]
files = 1
"#;

/// A task in `wip` at `difficulty` declaring `count` files.
fn judged(difficulty: Option<f64>, count: f64) -> Task {
    Task {
        difficulty: difficulty.map(crate::numeric::NumValue::Num),
        ..task("t", "wip", count)
    }
}

#[test]
fn a_gated_entry_caps_the_tasks_its_when_matches() {
    let problems = problems(&parse_config(GATED).unwrap(), &judged(Some(5.0), 6.0));
    assert_eq!(
        problems,
        vec!["6 files is over the 5 allowed when `difficulty >= 3`; judgment work stays small"],
        "the breach names the gate and carries the entry's message as the advice"
    );
}

#[test]
fn a_task_the_gate_skips_falls_through_to_the_next_entry() {
    let config = parse_config(GATED).unwrap();
    assert!(problems(&config, &judged(Some(1.0), 12.0)).is_empty());
    assert!(
        problems(&config, &judged(None, 12.0)).is_empty(),
        "a task declaring no difficulty is not matched by `difficulty >= 3`"
    );
    assert_eq!(
        problems(&config, &judged(Some(1.0), 16.0)),
        vec!["16 files is over the 15 allowed; split the task before starting the work"],
    );
}

#[test]
fn an_entry_bounds_only_the_units_it_names() {
    let mut long = judged(Some(5.0), 2.0);
    long.estimate.insert("hours".to_owned(), 50.0);
    assert_eq!(
        problems(&parse_config(GATED).unwrap(), &long),
        vec!["50 hours is over the 40 allowed; split the task before starting the work"],
        "the gated entry declares no `hours`, so the fallback's ceiling still binds"
    );
}

#[test]
fn a_min_entry_takes_a_when_and_a_message_too() {
    let toml = GATED.replace(
        "[[tasks.estimate.min]]\nfiles = 1",
        "[[tasks.estimate.min]]\nwhen = 'kind == \"umbrella\"'\nfiles = 3\nmessage = \"an umbrella spans leaves\"",
    );
    let config = parse_config(&format!(
        "{toml}\n[tasks.fields.kind]\nvalues = [\"umbrella\"]\n"
    ))
    .unwrap();
    let mut umbrella = judged(Some(1.0), 2.0);
    umbrella.kind = Some("umbrella".to_owned());
    assert_eq!(
        problems(&config, &umbrella),
        vec![
            "2 files is under the 3 required when `kind == \"umbrella\"`; an umbrella spans leaves"
        ],
    );
    assert!(problems(&config, &judged(Some(1.0), 0.0)).is_empty());
}

#[test]
fn a_gated_over_cap_estimate_fails_validation_and_a_terminal_one_does_not() {
    let config = parse_config(GATED).unwrap();
    let violations = validate(&[judged(Some(5.0), 6.0)], &config);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations.first().unwrap().field, "estimate");
    let shipped = Task {
        difficulty: Some(crate::numeric::NumValue::Num(5.0)),
        ..task("shipped", "done", 6.0)
    };
    assert!(validate(&[shipped], &config).is_empty());
}

/// The config-load error `toml` produces, as text: a parse failure, else the
/// semantic check `load_config` runs after it.
fn load_error(toml: &str) -> String {
    match parse_config(toml) {
        Err(err) => err.to_string(),
        Ok(config) => config.validate().err().unwrap_or_default(),
    }
}

#[test]
fn a_malformed_when_is_one_config_error_naming_the_entry() {
    let err = load_error(&GATED.replace("difficulty >= 3", "difficulty >="));
    assert!(err.contains("tasks.estimate.max[0].when"), "{err}");
    let err = load_error(&GATED.replace("difficulty >= 3", "dificulty >= 3"));
    assert!(err.contains("tasks.estimate.max[0].when"), "{err}");
}

#[test]
fn an_entry_bounding_no_unit_is_a_config_error() {
    let err = load_error(&GATED.replace(
        "[[tasks.estimate.min]]\nfiles = 1",
        "[[tasks.estimate.min]]\nmessage = \"nothing\"",
    ));
    assert!(err.contains("tasks.estimate.min[0]"), "{err}");
    assert!(err.contains("bounds no unit"), "{err}");
}

#[test]
fn a_bound_on_a_unit_outside_the_allow_list_is_a_config_error() {
    let err = load_error(&GATED.replace("hours = 40", "furlongs = 40"));
    assert!(
        err.contains("tasks.estimate.max[1].furlongs")
            && err.contains("not an allowed estimate unit"),
        "{err}"
    );
}

#[test]
fn a_bound_quantity_must_be_a_number() {
    let err = load_error(&GATED.replace("files = 15", "files = \"15\""));
    assert!(err.contains("files") || err.contains("f64"), "{err}");
}
