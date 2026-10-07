//! Field-scoped validation: a single field's required/scale/enum/format
//! constraints, in isolation from the cross-task concerns (status
//! reachability, link resolution, id uniqueness) covered in
//! `validate/tests.rs`. Shares that file's `config()`/`task()` fixtures.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::tests::{config, task};
use super::validate;
use crate::config::{Config, FieldConfig, Requirement, parse_config};
use crate::model::marker::ShapeError;
use crate::model::numeric::NumValue;
use crate::test_support::ice_priority;

#[test]
fn flags_missing_required_priority_when_active() {
    let mut item = task("a", "wip");
    item.priority = None;
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "priority"));
}

#[test]
fn terminal_task_skips_required() {
    let mut item = task("a", "done");
    item.priority = None;
    assert!(validate(&[item], &config()).is_empty());
}

#[test]
fn flags_priority_out_of_scale() {
    let mut item = task("a", "wip");
    item.priority = Some(NumValue::Num(42.0));
    let violations = validate(&[item], &config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "priority" && v.message.contains("outside")),
        "out-of-scale priority should be flagged, got: {violations:?}"
    );
}

#[test]
fn flags_unknown_priority_label() {
    let mut item = task("a", "wip");
    item.priority = Some(NumValue::Label("epic".to_owned()));
    let violations = validate(&[item], &config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "priority" && v.message.contains("not a known priority value")),
        "a label outside the configured bands should be flagged, got: {violations:?}"
    );
}

#[test]
fn flags_unknown_estimate_unit() {
    let mut item = task("a", "wip");
    item.estimate = BTreeMap::from([("fortnights".to_owned(), 2.0)]);
    let violations = validate(&[item], &config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "estimate" && v.message.contains("fortnights"))
    );
}

#[test]
fn terminal_task_keeps_a_retired_estimate_unit() {
    let mut item = task("a", "done");
    item.estimate = BTreeMap::from([("fortnights".to_owned(), 2.0)]);
    let violations = validate(&[item], &config());
    assert!(
        !violations.iter().any(|v| v.field == "estimate"),
        "a terminal task should keep whatever unit it was estimated in, got: {violations:?}"
    );
}

#[test]
fn flags_marker_shape_error_regardless_of_config() {
    let mut item = task("a", "wip");
    item.shape_errors = vec![ShapeError {
        field: "tags".to_owned(),
        message: "expected an array, found a string".to_owned(),
    }];
    let violations = validate(&[item], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "tags" && v.message.contains("expected an array"))
    );
}

#[test]
fn flags_disallowed_tag() {
    let mut item = task("a", "wip");
    item.tags = vec!["frontend".to_owned()];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "tags"));
}

#[test]
fn disallowed_tag_near_miss_offers_a_suggestion() {
    // `config()`'s `[tasks.tags] allowed` is `["backend"]`.
    let mut item = task("a", "wip");
    item.tags = vec!["backendd".to_owned()];
    let violations = validate(&[item], &config());
    let hit = violations
        .iter()
        .find(|v| v.field == "tags")
        .expect("disallowed tag must be flagged");
    assert!(hit.message.contains("did you mean: backend?"), "{hit:?}");
    assert!(hit.message.contains("known tags: backend"), "{hit:?}");
}

#[test]
fn disallowed_tag_far_miss_omits_the_suggestion() {
    let mut item = task("a", "wip");
    item.tags = vec!["frontend".to_owned()];
    let violations = validate(&[item], &config());
    let hit = violations
        .iter()
        .find(|v| v.field == "tags")
        .expect("disallowed tag must be flagged");
    assert!(!hit.message.contains("did you mean"), "{hit:?}");
    assert!(hit.message.contains("known tags: backend"), "{hit:?}");
}

#[test]
fn flags_disallowed_derived_tag() {
    let mut item = task("a", "wip");
    item.derived_tags = vec!["marketing".to_owned()];
    let violations = validate(&[item], &config());
    let hit = violations
        .iter()
        .find(|v| v.field == "tags")
        .expect("derived tag outside the allow-list must be flagged");
    assert!(hit.message.contains("derived from folder"));
    assert!(hit.message.contains("known tags: backend"), "{hit:?}");
}

#[test]
fn disallowed_derived_tag_near_miss_suggests_the_allowed_tag() {
    let mut item = task("a", "wip");
    item.derived_tags = vec!["backedn".to_owned()];
    let violations = validate(&[item], &config());
    let hit = violations
        .iter()
        .find(|v| v.field == "tags")
        .expect("derived tag outside the allow-list must be flagged");
    assert!(hit.message.contains("did you mean: backend?"), "{hit:?}");
}

#[test]
fn terminal_task_keeps_a_dropped_tag() {
    let mut item = task("a", "done");
    item.tags = vec!["frontend".to_owned()];
    let violations = validate(&[item], &config());
    assert!(
        !violations.iter().any(|v| v.field == "tags"),
        "a terminal task should keep whatever tag it was filed with, got: {violations:?}"
    );
}

#[test]
fn terminal_task_keeps_a_dropped_derived_tag() {
    let mut item = task("a", "done");
    item.derived_tags = vec!["marketing".to_owned()];
    let violations = validate(&[item], &config());
    assert!(
        !violations.iter().any(|v| v.field == "tags"),
        "a terminal task should keep whatever tag its folder derived, got: {violations:?}"
    );
}

/// A config with ICE active in strict mode (no manual integers).
fn strict_ice_config() -> Config {
    parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n\
         [tasks.priority]\nmin = 0\nmax = 9\nframework = \"ice\"\nallow-manual = false\n",
    )
    .unwrap()
}

#[test]
fn framework_priority_validates_clean() {
    let mut item = task("a", "wip");
    item.priority = Some(ice_priority(10.0, 0.9, 10.0));
    let violations = validate(&[item], &strict_ice_config());
    assert!(
        !violations.iter().any(|v| v.field == "priority"),
        "valid ICE inputs should pass, got: {violations:?}"
    );
}

#[test]
fn strict_mode_rejects_manual_integer() {
    let mut item = task("a", "wip");
    item.priority = Some(NumValue::Num(7.0));
    let violations = validate(&[item], &strict_ice_config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "priority" && v.message.contains("strict")),
        "strict mode should reject a manual int, got: {violations:?}"
    );
}

#[test]
fn rejects_inactive_framework() {
    let mut item = task("a", "wip");
    item.priority = Some(NumValue::Framework {
        name: "rice".to_owned(),
        inputs: BTreeMap::new(),
    });
    let violations = validate(&[item], &strict_ice_config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "priority" && v.message.contains("not the active framework")),
        "a non-active framework should be flagged, got: {violations:?}"
    );
}

#[test]
fn flags_out_of_bounds_framework_input() {
    let mut item = task("a", "wip");
    // impact 99 exceeds the declared 1..10 bound.
    item.priority = Some(ice_priority(99.0, 0.5, 5.0));
    let violations = validate(&[item], &strict_ice_config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "priority" && v.message.contains("impact")),
        "out-of-bounds input should be flagged, got: {violations:?}"
    );
}

#[test]
fn lenient_mode_accepts_manual_integer() {
    let cfg = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n\
         [tasks.priority]\nmin = 0\nmax = 9\nframework = \"ice\"\nallow-manual = true\n",
    )
    .unwrap();
    let mut item = task("a", "wip");
    item.priority = Some(NumValue::Num(7.0));
    let violations = validate(&[item], &cfg);
    assert!(
        !violations.iter().any(|v| v.field == "priority"),
        "lenient mode should accept a manual int, got: {violations:?}"
    );
}

fn kind_config() -> Config {
    let mut cfg = config();
    cfg.tasks.fields.insert(
        "kind".into(),
        FieldConfig {
            values: vec!["bug".into(), "feature".into()],
            required: Requirement::Expr("!terminal".to_owned()),
            ..FieldConfig::default()
        },
    );
    cfg
}

#[test]
fn kind_missing_required_flags_on_non_terminal() {
    let mut t = task("a", "wip");
    t.kind = None;
    assert!(
        validate(&[t], &kind_config())
            .iter()
            .any(|v| v.field == "kind")
    );
}

#[test]
fn kind_missing_required_skips_on_terminal() {
    let mut t = task("a", "done");
    t.kind = None;
    assert!(
        !validate(&[t], &kind_config())
            .iter()
            .any(|v| v.field == "kind")
    );
}

#[test]
fn kind_allowed_value_passes() {
    let mut t = task("a", "wip");
    t.kind = Some("feature".into());
    assert!(
        !validate(&[t], &kind_config())
            .iter()
            .any(|v| v.field == "kind")
    );
}

#[test]
fn kind_disallowed_value_flags() {
    let mut t = task("a", "wip");
    t.kind = Some("chore".into());
    assert!(
        validate(&[t], &kind_config())
            .iter()
            .any(|v| v.field == "kind")
    );
}

fn assignee_config() -> Config {
    let mut cfg = config();
    cfg.tasks.assignee.allowed = vec!["ada".into(), "lin".into()];
    cfg.tasks.assignee.required = Requirement::Expr("!terminal".to_owned());
    cfg
}

#[test]
fn assignee_missing_required_flags_on_non_terminal() {
    let mut t = task("a", "wip");
    t.assignee = None;
    let violations = validate(&[t], &assignee_config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "assignee" && v.message.contains("missing")),
        "{violations:?}"
    );
}

#[test]
fn assignee_missing_required_skips_on_terminal() {
    let mut t = task("a", "done");
    t.assignee = None;
    assert!(
        !validate(&[t], &assignee_config())
            .iter()
            .any(|v| v.field == "assignee")
    );
}

#[test]
fn assignee_allowed_value_passes() {
    let mut t = task("a", "wip");
    t.assignee = Some("ada".into());
    assert!(
        !validate(&[t], &assignee_config())
            .iter()
            .any(|v| v.field == "assignee")
    );
}

#[test]
fn assignee_disallowed_value_flags() {
    let mut t = task("a", "wip");
    t.assignee = Some("eve".into());
    let violations = validate(&[t], &assignee_config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "assignee" && v.message.contains("eve")),
        "{violations:?}"
    );
}

#[test]
fn difficulty_out_of_scale_flags() {
    let mut t = task("a", "wip");
    t.difficulty = Some(NumValue::Num(42.0));
    assert!(
        validate(&[t], &config())
            .iter()
            .any(|v| v.field == "difficulty")
    );
}

#[test]
fn difficulty_min_boundary_passes() {
    let mut t = task("a", "wip");
    t.difficulty = Some(NumValue::Num(0.0));
    assert!(
        !validate(&[t], &config())
            .iter()
            .any(|v| v.field == "difficulty")
    );
}

#[test]
fn difficulty_max_boundary_passes() {
    let mut t = task("a", "wip");
    t.difficulty = Some(NumValue::Num(9.0));
    assert!(
        !validate(&[t], &config())
            .iter()
            .any(|v| v.field == "difficulty")
    );
}

#[test]
fn flags_unknown_difficulty_label() {
    let mut t = task("a", "wip");
    t.difficulty = Some(NumValue::Label("expert".to_owned()));
    let violations = validate(&[t], &config());
    assert!(
        violations.iter().any(|v| {
            v.field == "difficulty" && v.message.contains("not a known difficulty value")
        }),
        "an unresolvable difficulty label should be flagged, got: {violations:?}"
    );
}

#[test]
fn priority_framework_score_outside_custom_scale_flags() {
    let mut t = task("a", "wip");
    t.priority = Some(ice_priority(10.0, 1.0, 10.0));
    let mut cfg = strict_ice_config();
    cfg.tasks.priority.field.min = 0.0;
    cfg.tasks.priority.field.max = 1.0;
    assert!(
        validate(&[t], &cfg)
            .iter()
            .any(|v| v.field == "priority" && v.message.contains("outside"))
    );
}

#[test]
fn priority_framework_min_boundary_passes() {
    let mut t = task("a", "wip");
    t.priority = Some(ice_priority(1.0, 0.0, 1.0));
    assert!(
        !validate(&[t], &strict_ice_config())
            .iter()
            .any(|v| v.field == "priority")
    );
}

#[test]
fn priority_framework_max_boundary_passes() {
    let mut t = task("a", "wip");
    t.priority = Some(ice_priority(10.0, 1.0, 10.0));
    assert!(
        !validate(&[t], &strict_ice_config())
            .iter()
            .any(|v| v.field == "priority")
    );
}
