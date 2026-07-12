#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::config::{Config, parse_config};
use crate::numeric::NumValue;
use crate::test_support::{bare_task, ice_priority};

use super::CanonicalTask;

#[test]
fn framework_priority_serializes_with_framework_inputs_and_score() {
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let mut task = bare_task("t1");
    task.priority = Some(ice_priority(8.0, 0.75, 4.0));
    let ct = CanonicalTask::from_task(&task, &config);
    let value = serde_json::to_value(&ct).unwrap();
    assert_eq!(value["priority"]["framework"], "ice");
    assert_eq!(value["priority"]["inputs"]["impact"], 8.0);
    assert_eq!(value["priority"]["inputs"]["confidence"], 0.75);
    assert_eq!(value["priority"]["inputs"]["ease"], 4.0);
    assert!(
        value["priority"]["score"].is_number(),
        "ice is registered, score must resolve"
    );
}

#[test]
fn framework_priority_score_is_null_outside_a_project() {
    // `Config::default()` carries no framework registry, so the score cannot
    // resolve — nullable, per the module doc.
    let config = Config::default();
    let mut task = bare_task("t1");
    task.priority = Some(ice_priority(8.0, 0.75, 4.0));
    let ct = CanonicalTask::from_task(&task, &config);
    let value = serde_json::to_value(&ct).unwrap();
    assert!(value["priority"]["score"].is_null());
}

#[test]
fn scalar_priority_serializes_as_bare_number() {
    let config = Config::default();
    let mut task = bare_task("t1");
    task.priority = Some(NumValue::Num(8.0));
    let ct = CanonicalTask::from_task(&task, &config);
    let value = serde_json::to_value(&ct).unwrap();
    assert_eq!(value["priority"], 8.0);
}

#[test]
fn body_serializes_non_empty() {
    let config = Config::default();
    let mut task = bare_task("t1");
    task.body = "== Summary\n\nSome text.".to_owned();
    let ct = CanonicalTask::from_task(&task, &config);
    assert!(!ct.body.is_empty());
    let value = serde_json::to_value(&ct).unwrap();
    assert_eq!(value["body"], "== Summary\n\nSome text.");
}

#[test]
fn round_trips_through_serde_json_value() {
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let mut task = bare_task("t1");
    task.title = "Build".to_owned();
    task.priority = Some(ice_priority(8.0, 0.75, 4.0));
    task.body = "notes".to_owned();
    task.tags = vec!["x".to_owned()];
    let ct = CanonicalTask::from_task(&task, &config);
    let value = serde_json::to_value(&ct).unwrap();
    let round_tripped: CanonicalTask = serde_json::from_value(value).unwrap();
    assert_eq!(ct, round_tripped);
}

#[test]
fn round_trips_a_scalar_priority_task() {
    let config = Config::default();
    let mut task = bare_task("t2");
    task.priority = Some(NumValue::Num(3.0));
    task.difficulty = Some(NumValue::Label("senior".to_owned()));
    let ct = CanonicalTask::from_task(&task, &config);
    let value = serde_json::to_value(&ct).unwrap();
    let round_tripped: CanonicalTask = serde_json::from_value(value).unwrap();
    assert_eq!(ct, round_tripped);
}
