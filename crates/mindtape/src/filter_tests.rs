#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::Filter;
use crate::model::{Status, Task};
use crate::numeric::NumValue;

fn sample() -> Task {
    Task {
        id: "add-auth".to_owned(),
        file: "tasks/add-auth.typ".into(),
        title: "Add Auth".to_owned(),
        status: Some(Status {
            name: "wip".to_owned(),
            date: Some("2026-06-14".to_owned()),
            note: None,
        }),
        priority: Some(NumValue::Num(8.0)),
        difficulty: Some(NumValue::Num(6.0)),
        kind: Some("feature".to_owned()),
        estimate: BTreeMap::from([("hours".to_owned(), 8.0)]),
        depends_on: Vec::new(),
        relates_to: vec!["other.typ".to_owned()],
        supersedes: Vec::new(),
        duplicates: Vec::new(),
        tags: vec!["backend".to_owned()],
        due: Some("2026-07-01".to_owned()),
        start: None,
        reviewed: None,
        extras: BTreeMap::new(),
    }
}

fn hit(expr: &str, task: &Task) -> bool {
    Filter::parse(expr).unwrap().matches(task).unwrap()
}

#[test]
fn equality_and_inequality() {
    let task = sample();
    assert!(hit("status == \"wip\"", &task));
    assert!(!hit("status == \"done\"", &task));
    assert!(hit("status != \"done\"", &task));
    assert!(hit("priority == 8", &task));
}

#[test]
fn numeric_fields_compare_as_numbers() {
    let task = sample();
    assert!(hit("priority > 5", &task));
    assert!(!hit("priority < 8", &task));
    assert!(hit("priority >= 8", &task));
    assert!(hit("difficulty == 6", &task));
    assert!(hit("difficulty <= 6", &task));
}

#[test]
fn estimate_is_presence_only() {
    let task = sample();
    assert!(hit("has(estimate)", &task));
    assert!(!hit("miss(estimate)", &task));
    // The opaque estimate dict has no scalar; comparison is rejected.
    assert!(
        Filter::parse("estimate > 4")
            .unwrap()
            .matches(&task)
            .is_err()
    );
}

#[test]
fn date_string_comparison() {
    let task = sample();
    assert!(hit("due < \"2026-08-01\"", &task));
    assert!(!hit("due > \"2026-08-01\"", &task));
}

#[test]
fn presence_functions() {
    let task = sample();
    assert!(hit("has(due)", &task));
    assert!(hit("miss(start)", &task));
    assert!(!hit("has(start)", &task));
    assert!(hit("has(tags)", &task));
    assert!(hit("has(relates_to)", &task));
}

#[test]
fn tag_and_text_functions() {
    let task = sample();
    assert!(hit("has_tag(\"backend\")", &task));
    assert!(!hit("has_tag(\"frontend\")", &task));
    assert!(hit("contains(title, \"auth\")", &task)); // case-insensitive
    assert!(hit("search(\"AUTH\")", &task));
}

#[test]
fn boolean_combinators() {
    let task = sample();
    assert!(hit("priority >= 7 && status != \"done\"", &task));
    assert!(hit("status == \"done\" || has_tag(\"backend\")", &task));
    assert!(hit("!has(start)", &task));
}

#[test]
fn missing_field_inequality_is_true() {
    let mut task = sample();
    task.status = None;
    assert!(hit("status != \"done\"", &task));
    assert!(!hit("status == \"done\"", &task));
}

#[test]
fn unknown_field_errors() {
    let task = sample();
    let result = Filter::parse("bogus == \"x\"").unwrap().matches(&task);
    assert!(result.is_err());
}

#[test]
fn parse_error_surfaces() {
    assert!(Filter::parse("(status == \"x\"").is_err());
}

#[test]
fn reviewed_field_comparable() {
    let mut task = sample();
    task.reviewed = Some("2026-01-01".to_owned());
    assert!(hit("reviewed < \"2026-06-01\"", &task));
    assert!(!hit("reviewed > \"2026-06-01\"", &task));
    assert!(hit("has(reviewed)", &task));
    assert!(!hit("miss(reviewed)", &task));
}

#[test]
fn stale_without_reviewed_uses_status_date() {
    let mut task = sample();
    // Use a clearly old status date so the test is not time-sensitive.
    if let Some(ref mut s) = task.status {
        s.date = Some("2020-01-01".to_owned());
    }
    assert!(hit("stale(1)", &task));
}

#[test]
fn stale_with_recent_reviewed_is_not_stale() {
    let mut task = sample();
    task.reviewed = Some(crate::date::today_iso());
    assert!(!hit("stale(1)", &task));
}

#[test]
fn stale_with_old_reviewed_is_stale() {
    let mut task = sample();
    task.reviewed = Some("2020-01-01".to_owned());
    assert!(hit("stale(1)", &task));
}

#[test]
fn stale_bad_arg_errors() {
    let task = sample();
    assert!(
        Filter::parse("stale(not_a_number)")
            .unwrap()
            .matches(&task)
            .is_err()
    );
}
