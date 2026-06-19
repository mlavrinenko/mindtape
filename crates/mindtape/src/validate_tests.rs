#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::validate;
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};

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

[tasks.fields.priority]
values = ["low", "high"]
required = "non-terminal"

[tasks.estimate]
units = ["hours", "days"]

[tasks.tags]
allowed = ["backend"]
"#,
    )
    .unwrap()
}

fn task(id: &str, status: &str) -> Task {
    Task {
        id: id.to_owned(),
        file: format!("tasks/{id}.typ").into(),
        title: id.to_owned(),
        status: Some(Status {
            name: status.to_owned(),
            date: None,
            note: None,
        }),
        priority: Some("high".to_owned()),
        grade: None,
        kind: None,
        estimate: BTreeMap::from([("hours".to_owned(), 8.0)]),
        depends_on: Vec::new(),
        relates_to: Vec::new(),
        supersedes: Vec::new(),
        duplicates: Vec::new(),
        tags: Vec::new(),
        due: None,
        start: None,
        reviewed: None,
        extras: BTreeMap::new(),
    }
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
