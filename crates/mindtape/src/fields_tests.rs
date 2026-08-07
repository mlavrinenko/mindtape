#![allow(clippy::unwrap_used)]

use super::{FIELDS, FieldValue, get};
use crate::config::{Config, parse_config};
use crate::model::Status;
use crate::numeric::NumValue;
use crate::test_support::bare_task;

fn config() -> Config {
    parse_config(
        r"
[tasks.priority]
min = 0
max = 9
",
    )
    .unwrap()
}

#[test]
fn get_resolves_every_registered_name() {
    let cfg = config();
    let task = bare_task("t");
    for def in FIELDS {
        assert!(get(def.name).is_some(), "{} must resolve", def.name);
        // Every accessor must run cleanly against a bare task (all fields unset).
        let _ = (def.get)(&task, &cfg);
    }
}

#[test]
fn unknown_name_is_none() {
    assert!(get("bogus").is_none());
}

#[test]
fn bare_task_fields_are_missing() {
    let cfg = config();
    let task = bare_task("t");
    for name in [
        "status",
        "priority",
        "difficulty",
        "kind",
        "due",
        "start",
        "reviewed",
        "assignee",
    ] {
        assert!(
            matches!((get(name).unwrap().get)(&task, &cfg), FieldValue::Missing),
            "{name} must be Missing on a bare task"
        );
    }
}

#[test]
fn title_and_id_are_never_missing() {
    let cfg = config();
    let task = bare_task("t");
    assert!(matches!(
        (get("title").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "t"
    ));
    assert!(matches!(
        (get("id").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "t"
    ));
}

#[test]
fn depth_resolves_numeric() {
    let cfg = config();
    let mut task = bare_task("t");
    task.depth = 3;
    assert!(matches!(
        (get("depth").unwrap().get)(&task, &cfg),
        FieldValue::Num(n) if n == 3.0
    ));
}

#[test]
fn assignee_and_reviewed_resolve_as_strings() {
    let cfg = config();
    let mut task = bare_task("t");
    task.assignee = Some("ada".to_owned());
    task.reviewed = Some("2026-08-01".to_owned());
    assert!(matches!(
        (get("assignee").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "ada"
    ));
    assert!(matches!(
        (get("reviewed").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "2026-08-01"
    ));
}

#[test]
fn status_resolves_the_legend_name() {
    let cfg = config();
    let mut task = bare_task("t");
    task.status = Some(Status {
        name: "wip".to_owned(),
        date: None,
        note: None,
    });
    assert!(matches!(
        (get("status").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "wip"
    ));
}

#[test]
fn priority_raw_number_resolves_numeric() {
    let cfg = config();
    let mut task = bare_task("t");
    task.priority = Some(NumValue::Num(7.0));
    assert!(matches!(
        (get("priority").unwrap().get)(&task, &cfg),
        FieldValue::Num(n) if n == 7.0
    ));
}

#[test]
fn priority_label_falls_back_to_string() {
    let cfg = config();
    let mut task = bare_task("t");
    task.priority = Some(NumValue::Label("high".to_owned()));
    assert!(matches!(
        (get("priority").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "high"
    ));
}
