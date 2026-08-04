//! Field-scoped validation: a single field's required/scale/enum/format
//! constraints, in isolation from the cross-task concerns (status
//! reachability, link resolution, id uniqueness) covered in
//! `validate_tests.rs`. Shares that file's `config()`/`task()` fixtures.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::tests::{config, task};
use super::validate;
use crate::config::{Config, FieldConfig, Requirement, parse_config};
use crate::model::ShapeError;
use crate::numeric::NumValue;
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
fn flags_disallowed_derived_tag() {
    let mut item = task("a", "wip");
    item.derived_tags = vec!["marketing".to_owned()];
    let violations = validate(&[item], &config());
    let hit = violations
        .iter()
        .find(|v| v.field == "tags")
        .expect("derived tag outside the allow-list must be flagged");
    assert!(hit.message.contains("derived from folder"));
}

#[test]
fn default_config_accepts_plain_string_id() {
    let mut t = task("my-task", "wip");
    t.priority = None; // remove priority so the only check is id
    // default config has no priority constraints
    let violations = validate(&[t], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "id"));
}

#[test]
fn uuid_v4_config_accepts_valid_v4() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v4\"\n").unwrap();
    let mut t = task("550e8400-e29b-41d4-a716-446655440000", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(!violations.iter().any(|v| v.field == "id"));
}

#[test]
fn uuid_v4_config_rejects_non_v4() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v4\"\n").unwrap();
    // This is a v7 UUID
    let mut t = task("018f4e2a-0000-7000-8000-000000000000", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "v7 id should be rejected by uuid-v4 format"
    );
}

#[test]
fn uuid_v4_config_rejects_non_uuid_string() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v4\"\n").unwrap();
    let mut t = task("my-plain-task", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "plain string rejected by uuid-v4 format"
    );
}

#[test]
fn uuid_v7_config_accepts_valid_v7() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v7\"\n").unwrap();
    // UUID v7: version nibble = 7
    let mut t = task("018f4e2a-0000-7000-8000-000000000000", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(!violations.iter().any(|v| v.field == "id"));
}

#[test]
fn uuid_v7_rejects_v4() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v7\"\n").unwrap();
    let mut t = task("550e8400-e29b-41d4-a716-446655440000", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "v4 id should be rejected by uuid-v7 format"
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
            default: None,
            required: Requirement::Expr("!terminal".to_owned()),
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
