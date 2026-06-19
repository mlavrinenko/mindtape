#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{default_order, priority_value, status_index};
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::numeric::NumValue;

fn config() -> Config {
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
"#,
    )
    .unwrap()
}

fn task(id: &str, status: &str, priority: f64, due: Option<&str>) -> Task {
    Task {
        id: id.to_owned(),
        file: format!("tasks/{id}.typ").into(),
        title: id.to_owned(),
        status: Some(Status {
            name: status.to_owned(),
            date: None,
            note: None,
        }),
        priority: Some(NumValue::Num(priority)),
        difficulty: None,
        kind: None,
        estimate: BTreeMap::new(),
        depends_on: Vec::new(),
        relates_to: Vec::new(),
        supersedes: Vec::new(),
        duplicates: Vec::new(),
        tags: Vec::new(),
        due: due.map(ToOwned::to_owned),
        start: None,
        reviewed: None,
        extras: BTreeMap::new(),
    }
}

#[test]
fn status_index_follows_legend_order() {
    let cfg = config();
    assert_eq!(status_index(&task("a", "proposed", 5.0, None), &cfg), 0);
    assert_eq!(status_index(&task("a", "wip", 5.0, None), &cfg), 1);
    assert_eq!(status_index(&task("a", "done", 5.0, None), &cfg), 2);
}

#[test]
fn default_order_sorts_status_then_priority_desc() {
    let cfg = config();
    let mut tasks = vec![
        task("low-wip", "wip", 2.0, None),
        task("high-wip", "wip", 8.0, None),
        task("any-proposed", "proposed", 1.0, None),
    ];
    default_order(&mut tasks, &cfg);
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    // proposed (index 0) leads; within wip, priority 8 before 2.
    assert_eq!(ids, vec!["any-proposed", "high-wip", "low-wip"]);
}

#[test]
fn default_order_breaks_priority_ties_by_due_ascending() {
    let cfg = config();
    let mut tasks = vec![
        task("late", "wip", 5.0, Some("2026-09-01")),
        task("early", "wip", 5.0, Some("2026-07-01")),
        task("none", "wip", 5.0, None),
    ];
    default_order(&mut tasks, &cfg);
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["early", "late", "none"]);
}

#[test]
fn priority_value_resolves_through_config() {
    let cfg = config();
    let item = task("a", "wip", 7.0, None);
    assert_eq!(priority_value(&item, &cfg), Some(7.0));
}
