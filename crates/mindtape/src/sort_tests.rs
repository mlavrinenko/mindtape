#![allow(clippy::unwrap_used)]

use super::{default_order, default_order_with, sort_by_specs, status_index, validate_sort_spec};
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::numeric::NumValue;
use crate::test_support::{bare_task, dep_edge};

/// Status legend and priority/difficulty scales shared by every test in this
/// file and its `sort_compare_tests.rs` sibling (which imports them via
/// `use super::tests::{config, task}`).
pub(super) fn config() -> Config {
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

[tasks.difficulty]
min = 0
max = 9
"#,
    )
    .unwrap()
}

/// A task with status, priority, and optional due set — shared with the
/// `sort_compare_tests.rs` sibling.
pub(super) fn task(id: &str, status: &str, priority: f64, due: Option<&str>) -> Task {
    let mut t = bare_task(id);
    t.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    t.priority = Some(NumValue::Num(priority));
    t.due = due.map(ToOwned::to_owned);
    t
}

#[test]
fn status_index_follows_legend_order() {
    let cfg = config();
    assert_eq!(status_index(&task("a", "proposed", 5.0, None), &cfg), 0);
    assert_eq!(status_index(&task("a", "wip", 5.0, None), &cfg), 1);
    assert_eq!(status_index(&task("a", "done", 5.0, None), &cfg), 2);
}

#[test]
fn default_order_sorts_priority_desc_then_status() {
    let cfg = config();
    let mut tasks = vec![
        task("low-wip", "wip", 2.0, None),
        task("high-wip", "wip", 8.0, None),
        task("any-proposed", "proposed", 1.0, None),
    ];
    default_order(&mut tasks, &cfg);
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    // Priority leads after readiness: 8, then 2, then 1 — the lower-priority
    // proposed task trails despite leading the lifecycle.
    assert_eq!(ids, vec!["high-wip", "low-wip", "any-proposed"]);
}

#[test]
fn default_order_breaks_priority_ties_by_status() {
    let cfg = config();
    // Equal priority: the lifecycle index breaks the tie (proposed before wip).
    let mut tasks = vec![
        task("the-wip", "wip", 5.0, None),
        task("the-proposed", "proposed", 5.0, None),
    ];
    default_order(&mut tasks, &cfg);
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["the-proposed", "the-wip"]);
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
fn default_order_floats_ready_above_blocked() {
    use crate::ready::Readiness;

    let cfg = config(); // proposed(0), wip(1), done(2)
    // A blocked task earlier in the lifecycle vs a ready task later in it: the
    // readiness band leads, so the ready task sorts first despite its status.
    let mut blocked = task("blocked-proposed", "proposed", 8.0, None);
    blocked.links = vec![dep_edge("missing.typ")];
    let ready = task("ready-wip", "wip", 1.0, None);

    let mut tasks = vec![blocked, ready];
    let readiness = Readiness::compute(&tasks, &cfg);
    default_order_with(&mut tasks, &cfg, &readiness);
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["ready-wip", "blocked-proposed"]);
}

#[test]
fn validate_sort_spec_accepts_all_fields() {
    for &field in super::SORT_FIELDS.iter() {
        assert!(
            validate_sort_spec(field).is_ok(),
            "expected {field} to be valid"
        );
        assert!(validate_sort_spec(&format!("{field}:asc")).is_ok());
        assert!(validate_sort_spec(&format!("{field}:desc")).is_ok());
    }
}

#[test]
fn validate_sort_spec_rejects_unknown_field() {
    assert!(validate_sort_spec("no_such_field").is_err());
    assert!(validate_sort_spec("no_such_field:desc").is_err());
}

#[test]
fn sort_by_specs_orders_by_id_ascending() {
    let cfg = config();
    let mut tasks = vec![
        task("z-task", "wip", 5.0, None),
        task("a-task", "wip", 5.0, None),
    ];
    sort_by_specs(
        &mut tasks,
        &["id".to_owned()],
        &cfg,
        &crate::ready::Readiness::empty(),
    )
    .unwrap();
    assert_eq!(tasks.first().unwrap().id, "a-task");
}

#[test]
fn sort_by_specs_rejects_unknown_field() {
    let cfg = config();
    let mut tasks = vec![task("a", "wip", 5.0, None)];
    assert!(
        sort_by_specs(
            &mut tasks,
            &["bogus".to_owned()],
            &cfg,
            &crate::ready::Readiness::empty()
        )
        .is_err()
    );
}

#[test]
fn default_order_with_honours_config_default_sort() {
    let cfg = parse_config(
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

[ls]
default-sort = ["id"]
"#,
    )
    .unwrap();
    // When default_sort = ["id"], tasks sort by id ascending — overriding the
    // built-in priority-first chain.
    let mut tasks = vec![
        task("z-task", "wip", 9.0, None),
        task("a-task", "wip", 1.0, None),
    ];
    default_order_with(&mut tasks, &cfg, &crate::ready::Readiness::empty());
    assert_eq!(tasks.first().unwrap().id, "a-task");
}

#[test]
fn sort_fields_covers_every_registry_field() {
    // A field wired into `crate::fields` (and therefore filterable) must also
    // be listed in `SORT_FIELDS` — otherwise it stays silently unsortable, the
    // exact drift this registry closes off. Add the field here (and, if its
    // sort order needs to differ from the generic registry compare, a
    // `compare_field` arm) when this fails.
    for def in crate::fields::FIELDS {
        assert!(
            super::SORT_FIELDS.contains(&def.name),
            "field {:?} is filterable but missing from SORT_FIELDS",
            def.name
        );
    }
}
