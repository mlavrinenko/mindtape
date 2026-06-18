#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::validate;
use crate::config::parse_config;
use crate::model::{Status, Task};

fn config() -> crate::config::Config {
    parse_config(
        r#"
[tasks.status]
values = ["proposed", "wip", "done"]
terminal = ["done"]

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
        tags: Vec::new(),
        due: None,
        start: None,
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
    item.depends_on = vec!["missing".to_owned()];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "depends-on"));
}

#[test]
fn resolves_existing_link() {
    let mut first = task("a", "wip");
    first.relates_to = vec!["b".to_owned()];
    let tasks = vec![first, task("b", "wip")];
    assert!(
        !validate(&tasks, &config())
            .iter()
            .any(|v| v.field == "relates-to")
    );
}
