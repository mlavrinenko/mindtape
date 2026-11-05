#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::config::{Config, parse_config};
use crate::estimate_queries::CountEstimate;
use crate::model::numeric::NumValue;
use crate::query::Query;
use crate::recur::{CanonicalRule, PeriodUnit, Recur, Rule};
use crate::test_support::{bare_task, ice_priority};

use super::{CanonicalCount, CanonicalNum, CanonicalTask, SCHEMA_VERSION};

/// The `ast-grep` query tag is why the canonical schema is version 2: an
/// importer older than the tag would fail in its enum instead of the version
/// check. `truth/tasks/bump-the-canonical-schema-for-the-ast-grep-tag.typ`.
#[test]
fn schema_version_is_two_for_the_ast_grep_query_tag() {
    assert_eq!(SCHEMA_VERSION, 2);
}

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

#[test]
fn minimal_title_only_record_deserializes() {
    let json = serde_json::json!({"title": "Fix login"});
    let task: CanonicalTask = serde_json::from_value(json).unwrap();
    assert_eq!(task.title, "Fix login");
    assert!(task.id.is_empty());
    assert!(task.file.is_empty());
    assert!(task.estimate.is_empty());
    assert!(task.links.is_empty());
    assert!(task.tags.is_empty());
    assert!(task.derived_tags.is_empty());
    assert!(task.body.is_empty());
    assert!(task.extras.is_empty());
    assert!(task.status.is_none());
    assert!(task.recur.is_none());
}

#[test]
fn every_recur_serializes_and_round_trips() {
    let config = Config::default();
    let mut task = bare_task("recur-every");
    task.recur = Some(Recur {
        rule: Rule::Period {
            unit: PeriodUnit::Weeks,
            value: 2,
            weekdays: vec![chrono::Weekday::Mon, chrono::Weekday::Wed],
            start: None,
        },
        anchor: Some("due".to_owned()),
        until: Some("2027-01-01".to_owned()),
        count: Some(4),
        materialize: Some("mutate".to_owned()),
    });
    let ct = CanonicalTask::from_task(&task, &config);
    let value = serde_json::to_value(&ct).unwrap();
    assert_eq!(value["recur"]["rule"]["kind"], "period");
    assert_eq!(value["recur"]["rule"]["unit"], "weeks");
    assert_eq!(value["recur"]["rule"]["value"], 2);
    assert_eq!(
        value["recur"]["rule"]["on"],
        serde_json::json!(["mon", "wed"])
    );
    assert_eq!(value["recur"]["anchor"], "due");
    assert_eq!(value["recur"]["until"], "2027-01-01");
    assert_eq!(value["recur"]["count"], 4);
    assert_eq!(value["recur"]["materialize"], "mutate");
    let round_tripped: CanonicalTask = serde_json::from_value(value).unwrap();
    assert_eq!(ct, round_tripped);
}

#[test]
fn rrule_recur_serializes_and_round_trips() {
    let config = Config::default();
    let mut task = bare_task("recur-rrule");
    task.recur = Some(Recur {
        rule: Rule::Rrule {
            raw: "FREQ=MONTHLY;BYDAY=1MO".to_owned(),
        },
        anchor: None,
        until: None,
        count: None,
        materialize: None,
    });
    let ct = CanonicalTask::from_task(&task, &config);
    assert_eq!(
        ct.recur.as_ref().unwrap().rule,
        CanonicalRule::Rrule {
            raw: "FREQ=MONTHLY;BYDAY=1MO".to_owned(),
        }
    );
    let value = serde_json::to_value(&ct).unwrap();
    assert_eq!(value["recur"]["rule"]["kind"], "rrule");
    assert_eq!(value["recur"]["rule"]["raw"], "FREQ=MONTHLY;BYDAY=1MO");
    let round_tripped: CanonicalTask = serde_json::from_value(value).unwrap();
    assert_eq!(ct, round_tripped);
}

#[test]
fn non_recurring_task_serializes_recur_as_null() {
    let config = Config::default();
    let task = bare_task("t1");
    let ct = CanonicalTask::from_task(&task, &config);
    let value = serde_json::to_value(&ct).unwrap();
    assert!(value["recur"].is_null());
}

#[test]
fn difficulty_projects_into_canonical_output() {
    // num_to_canonical must carry the authored difficulty through; a stub that
    // dropped it (returning None) would leave `difficulty` null. The round-trip
    // tests miss this — both sides collapse to None symmetrically.
    let config = Config::default();
    let mut task = bare_task("t1");
    task.difficulty = Some(NumValue::Label("senior".to_owned()));
    let ct = CanonicalTask::from_task(&task, &config);
    assert_eq!(
        ct.difficulty,
        Some(CanonicalNum::Label("senior".to_owned()))
    );
    let value = serde_json::to_value(&ct).unwrap();
    assert_eq!(value["difficulty"], "senior");
}

#[test]
fn children_and_progress_serialize_as_count_and_fraction() {
    let mut task = bare_task("epic");
    task.children = 5;
    task.children_done = 3;
    let value = serde_json::to_value(CanonicalTask::from_task(&task, &Config::default())).unwrap();
    assert_eq!(value["children"], 5);
    assert_eq!(value["progress"], 0.6);
    let leaf = serde_json::to_value(CanonicalTask::from_task(
        &bare_task("leaf"),
        &Config::default(),
    ))
    .unwrap();
    assert_eq!(leaf["children"], 0);
    assert!(leaf["progress"].is_null());
}

#[test]
fn an_ast_grep_query_serializes_with_the_hyphenated_tag_and_round_trips() {
    let count = CountEstimate {
        manual: 1.0,
        queries: vec![Query::AstGrep {
            language: "rust".to_owned(),
            pattern: "parse_task($$$ARGS)".to_owned(),
            glob: Some("crates/**".to_owned()),
        }],
        matches: vec![3],
        problem: None,
    };
    let canonical = CanonicalCount::from_count(&count);
    let value = serde_json::to_value(&canonical).unwrap();
    assert_eq!(value["queries"][0]["query"], "ast-grep");
    assert_eq!(value["queries"][0]["language"], "rust");
    assert_eq!(value["queries"][0]["pattern"], "parse_task($$$ARGS)");
    assert_eq!(value["queries"][0]["glob"], "crates/**");
    assert_eq!(value["queries"][0]["files"], 3);
    let back = canonical.to_count();
    assert_eq!(back.manual, count.manual);
    assert_eq!(back.queries, count.queries);
}
