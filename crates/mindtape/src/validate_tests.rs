#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::validate;
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::numeric::{InputValue, NumValue};
use crate::test_support::bare_task;

fn config() -> crate::config::Config {
    parse_config(
        r#"
[[tasks.status]]
value = "proposed"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.priority]
min = 0
max = 9
required = "non-terminal"
[[tasks.priority.bands]]
label = "low"
min = 0
max = 4
[[tasks.priority.bands]]
label = "high"
min = 5
max = 9

[tasks.estimate]
units = ["hours", "days"]

[tasks.tags]
allowed = ["backend"]
"#,
    )
    .unwrap()
}

fn task(id: &str, status: &str) -> Task {
    let mut t = bare_task(id);
    t.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    t.priority = Some(NumValue::Num(8.0));
    t.estimate = BTreeMap::from([("hours".to_owned(), 8.0)]);
    t
}

#[test]
fn clean_task_has_no_violations() {
    let tasks = vec![task("a", "wip")];
    assert!(validate(&tasks, &config()).is_empty());
}

#[test]
fn flags_unknown_status() {
    let tasks = vec![task("a", "bogus")];
    let violations = validate(&tasks, &config());
    assert!(violations.iter().any(|v| v.field == "status"));
}

#[test]
fn flags_dependency_cycle() {
    let mut a = task("a", "wip");
    let mut b = task("b", "wip");
    a.depends_on = vec!["b.typ".to_owned()];
    b.depends_on = vec!["a.typ".to_owned()];
    let violations = validate(&[a, b], &config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "depends-on" && v.message.contains("cycle"))
    );
}

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
fn flags_disallowed_tag() {
    let mut item = task("a", "wip");
    item.tags = vec!["frontend".to_owned()];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "tags"));
}

#[test]
fn flags_unresolved_link() {
    let mut item = task("a", "wip");
    item.depends_on = vec!["missing.typ".to_owned()];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "depends-on"));
}

#[test]
fn flags_unresolved_supersedes() {
    let mut item = task("a", "wip");
    item.supersedes = vec!["gone.typ".to_owned()];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "supersedes"));
}

#[test]
fn flags_unresolved_duplicates() {
    let mut item = task("a", "wip");
    item.duplicates = vec!["ghost.typ".to_owned()];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "duplicates"));
}

#[test]
fn resolves_supersedes_and_duplicates() {
    let mut first = task("a", "wip");
    // sibling file: tasks/b.typ — resolvable from tasks/a.typ as "b.typ"
    first.supersedes = vec!["b.typ".to_owned()];
    first.duplicates = vec!["b.typ".to_owned()];
    let tasks = vec![first, task("b", "wip")];
    let violations = validate(&tasks, &config());
    assert!(!violations.iter().any(|v| v.field == "supersedes"));
    assert!(!violations.iter().any(|v| v.field == "duplicates"));
}

#[test]
fn resolves_existing_link() {
    let mut first = task("a", "wip");
    // sibling file: tasks/b.typ — resolvable from tasks/a.typ as "b.typ"
    first.relates_to = vec!["b.typ".to_owned()];
    let tasks = vec![first, task("b", "wip")];
    assert!(
        !validate(&tasks, &config())
            .iter()
            .any(|v| v.field == "relates-to")
    );
}

#[test]
fn broken_link_violation_names_resolved_path() {
    let mut item = task("a", "wip");
    item.depends_on = vec!["nonexistent.typ".to_owned()];
    let violations = validate(&[item], &config());
    let v = violations
        .iter()
        .find(|v| v.field == "depends-on")
        .expect("expected a depends-on violation");
    assert!(
        v.message.contains("tasks/nonexistent.typ"),
        "violation message should name resolved path, got: {}",
        v.message
    );
}

#[test]
fn link_across_nested_directories_resolves() {
    // Task a lives at sub/a.typ; task b lives at other/b.typ.
    // From sub/a.typ, "b.typ" in other/ is reached via "../other/b.typ".
    let mut a = task("a", "wip");
    a.file = "sub/a.typ".into();
    a.depends_on = vec!["../other/b.typ".to_owned()];

    let mut b = task("b", "wip");
    b.file = "other/b.typ".into();

    let tasks = vec![a, b];
    let violations = validate(&tasks, &config());
    assert!(
        !violations.iter().any(|v| v.field == "depends-on"),
        "cross-directory link should resolve, got: {violations:?}"
    );
}

#[test]
fn explicit_id_task_still_reachable_by_path() {
    // A task whose id differs from its stem is still found as a link target
    // because resolution is now path-based, not id-based.
    let mut target = task("stem", "wip");
    target.id = "custom-uuid-style-id".to_owned();
    target.file = "tasks/stem.typ".into();

    let mut linker = task("linker", "wip");
    linker.file = "tasks/linker.typ".into();
    linker.depends_on = vec!["stem.typ".to_owned()];

    let tasks = vec![linker, target];
    let violations = validate(&tasks, &config());
    assert!(
        !violations.iter().any(|v| v.field == "depends-on"),
        "task with explicit id should still resolve by file path, got: {violations:?}"
    );
}

#[test]
fn broken_link_across_nested_directories_flagged() {
    let mut item = task("a", "wip");
    item.file = "sub/a.typ".into();
    item.depends_on = vec!["../missing/nowhere.typ".to_owned()];
    let violations = validate(&[item], &config());
    assert!(
        violations.iter().any(|v| v.field == "depends-on"),
        "non-existent cross-directory link should be flagged"
    );
}

#[test]
fn flags_duplicate_ids() {
    let mut second = task("a", "wip");
    second.file = "tasks/a2.typ".into();
    let tasks = vec![task("a", "wip"), second];
    let violations = validate(&tasks, &Config::default());
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "expected uniqueness violation"
    );
}

#[test]
fn no_uniqueness_violation_for_distinct_ids() {
    let tasks = vec![task("a", "wip"), task("b", "wip")];
    assert!(
        !validate(&tasks, &Config::default())
            .iter()
            .any(|v| v.field == "id")
    );
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

/// A config with ICE active in strict mode (no manual integers).
fn strict_ice_config() -> Config {
    parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n\
         [tasks.priority]\nmin = 0\nmax = 9\nframework = \"ice\"\nallow-manual = false\n",
    )
    .unwrap()
}

fn ice_priority(impact: f64, confidence: f64, ease: f64) -> NumValue {
    NumValue::Framework {
        name: "ice".to_owned(),
        inputs: BTreeMap::from([
            ("impact".to_owned(), InputValue::Num(impact)),
            ("confidence".to_owned(), InputValue::Num(confidence)),
            ("ease".to_owned(), InputValue::Num(ease)),
        ]),
    }
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
