//! Where the shape-error channel meets the `required` rules: a field the
//! marker could not read is reported once, naming the type, and never a second
//! time as missing. Split out of `validate/tests_fields.rs` (file-size gate),
//! whose subject is what a single field's constraints say about a value that
//! DID load.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::tests::config;
use super::{Violation, validate};
use crate::config::{Config, FieldConfig, Requirement};
use crate::model::marker::ShapeError;
use crate::model::numeric::NumValue;
use crate::model::{Status, Task};
use crate::test_support::bare_task;

/// Every core field a shape error can empty, so a mask covering only some of
/// them fails loudly rather than passing on the one case a test happened to pick.
const EMPTIABLE: [&str; 6] = [
    "status",
    "priority",
    "difficulty",
    "estimate",
    "assignee",
    "kind",
];

/// A config requiring all of [`EMPTIABLE`]: the shared legend and priority
/// scale, plus `difficulty`, `estimate`, `assignee` and a `kind` enum field all
/// made mandatory.
fn demanding_config() -> Config {
    let mut cfg = config();
    let always = Requirement::Always;
    cfg.tasks.difficulty.required = always.clone();
    cfg.tasks.estimate.required = always.clone();
    cfg.tasks.assignee.required = always.clone();
    cfg.tasks.priority.field.required = always.clone();
    cfg.tasks.fields.insert(
        "kind".into(),
        FieldConfig {
            values: vec!["bug".into(), "feature".into()],
            required: always,
            ..FieldConfig::default()
        },
    );
    cfg
}

/// A task carrying every required field with a legal value — the clean baseline
/// each case below breaks in exactly one place.
fn complete_task() -> Task {
    let mut t = bare_task("a");
    t.status = Some(Status {
        name: "wip".to_owned(),
        date: None,
        note: None,
    });
    t.priority = Some(NumValue::Num(8.0));
    t.difficulty = Some(NumValue::Num(3.0));
    t.estimate = BTreeMap::from([("hours".to_owned(), 8.0)]);
    t.assignee = Some("ada".to_owned());
    t.kind = Some("bug".to_owned());
    t
}

/// [`complete_task`] with `name` emptied the way `Task::from_marker` empties a
/// field it could not read.
fn emptied(name: &str) -> Task {
    let mut t = complete_task();
    match name {
        "status" => t.status = None,
        "priority" => t.priority = None,
        "difficulty" => t.difficulty = None,
        "estimate" => t.estimate = BTreeMap::new(),
        "assignee" => t.assignee = None,
        _ => t.kind = None,
    }
    t
}

/// [`emptied`], plus the [`ShapeError`] the projection would have left at
/// `path` — the field name for a field-level report, an element path for an
/// element one.
fn unreadable(name: &str, path: &str, message: &str) -> Task {
    let mut t = emptied(name);
    t.shape_errors = vec![ShapeError {
        field: path.to_owned(),
        message: message.to_owned(),
    }];
    t
}

/// The one violation `validate` returned. Every case here is about the COUNT as
/// much as the content — the bug WAS a second report — so the assertion is
/// shared rather than restated per case, and it is what keeps the read
/// index-free under `clippy::indexing_slicing`.
fn only(violations: &[Violation], what: &str) -> Violation {
    assert_eq!(
        violations.len(),
        1,
        "{what} must cost exactly one report, got: {violations:?}"
    );
    violations
        .first()
        .expect("a length of one was just asserted")
        .clone()
}

/// Each violation's field path, in order, for a case whose subject is which
/// reports survived rather than how they are worded.
fn paths(violations: &[Violation]) -> Vec<&str> {
    violations.iter().map(|v| v.field.as_str()).collect()
}

#[test]
fn wrong_typed_required_status_reports_the_type_and_nothing_else() {
    let task = unreadable(
        "status",
        "status",
        "expected a status like `wip(2026, 6, 14)`, found a string",
    );
    let violations = validate(&[task], &demanding_config());
    let hit = only(&violations, "`status: \"wip\"`");
    assert_eq!(hit.field, "status");
    assert!(
        hit.message.contains("found a string"),
        "the surviving report must name the type, got: {hit:?}"
    );
}

#[test]
fn every_wrong_typed_required_field_reports_only_the_type() {
    for name in EMPTIABLE {
        let task = unreadable(name, name, "expected a shape, found a boolean");
        let hit = only(&validate(&[task], &demanding_config()), name);
        assert!(
            hit.message.contains("found a boolean"),
            "`{name}`'s surviving report must be the shape one, got: {hit:?}"
        );
    }
}

#[test]
fn a_genuinely_absent_required_field_still_reports_missing() {
    for name in EMPTIABLE {
        let hit = only(&validate(&[emptied(name)], &demanding_config()), name);
        assert_eq!(hit.field, name);
        assert!(
            hit.message.contains("missing"),
            "`{name}` must be reported as missing, got: {hit:?}"
        );
    }
}

#[test]
fn an_element_error_still_leaves_the_required_check_to_fire() {
    // `estimate: (days: "two")` — the field read as a dict, one entry did not.
    // The estimate that survives is empty, and "required field is missing" is a
    // true statement about it, so BOTH reports are correct here.
    let task = unreadable(
        "estimate",
        "estimate.days",
        "expected a number, found a string",
    );
    let violations = validate(&[task], &demanding_config());
    assert_eq!(
        paths(&violations),
        ["estimate.days", "estimate"],
        "{violations:?}"
    );
}

#[test]
fn an_element_error_does_not_mask_a_sibling_required_field() {
    // `status.date` leaves `status` itself readable, so the mask must key on the
    // exact path and never on its leading field name.
    let mut task = emptied("priority");
    task.shape_errors = vec![ShapeError {
        field: "status.date".to_owned(),
        message: "expected a string, found a number".to_owned(),
    }];
    let violations = validate(&[task], &demanding_config());
    assert_eq!(
        paths(&violations),
        ["status.date", "priority"],
        "{violations:?}"
    );
}

#[test]
fn a_shape_error_on_one_field_does_not_mask_another() {
    let mut task = unreadable("priority", "priority", "expected a number, found a boolean");
    task.assignee = None;
    let violations = validate(&[task], &demanding_config());
    assert_eq!(
        paths(&violations),
        ["priority", "assignee"],
        "{violations:?}"
    );
}
