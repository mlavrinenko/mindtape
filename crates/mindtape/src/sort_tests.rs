#![allow(clippy::unwrap_used)]

use super::{
    default_order, default_order_with, priority_value, sort_by_specs, status_index,
    validate_sort_spec,
};
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::numeric::NumValue;
use crate::test_support::{bare_task, dep_edge};

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

[tasks.difficulty]
min = 0
max = 9
"#,
    )
    .unwrap()
}

/// Ids after sorting `tasks` by a single `field` spec (no readiness signal).
fn ids_by(mut tasks: Vec<Task>, field: &str, cfg: &Config) -> Vec<String> {
    sort_by_specs(
        &mut tasks,
        &[field.to_owned()],
        cfg,
        &crate::ready::Readiness::empty(),
    )
    .unwrap();
    tasks.iter().map(|t| t.id.clone()).collect()
}

fn task(id: &str, status: &str, priority: f64, due: Option<&str>) -> Task {
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
fn priority_value_resolves_through_config() {
    let cfg = config();
    let item = task("a", "wip", 7.0, None);
    assert_eq!(priority_value(&item, &cfg), Some(7.0));
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

// Per-field `--sort` comparators: each `compare_field` / `sort_string_key` arm
// must route to its own key. Input order is the inverse of the expected sorted
// order, so dropping an arm (which collapses to an equal/empty key and leaves
// the input order) fails the assertion.

#[test]
fn sort_by_specs_orders_by_status_lifecycle() {
    let cfg = config();
    let tasks = vec![
        task("s-done", "done", 5.0, None),
        task("s-proposed", "proposed", 5.0, None),
    ];
    assert_eq!(ids_by(tasks, "status", &cfg), vec!["s-proposed", "s-done"]);
}

#[test]
fn sort_by_specs_orders_by_priority_ascending() {
    let cfg = config();
    let tasks = vec![
        task("p-high", "wip", 8.0, None),
        task("p-low", "wip", 2.0, None),
    ];
    assert_eq!(ids_by(tasks, "priority", &cfg), vec!["p-low", "p-high"]);
}

#[test]
fn sort_by_specs_orders_by_difficulty_ascending() {
    let cfg = config();
    // Distinct resolved difficulties, input hard-first: a constant or absent
    // `difficulty_value` would leave the input order and fail this.
    let mut hard = bare_task("d-hard");
    hard.difficulty = Some(NumValue::Num(8.0));
    let mut easy = bare_task("d-easy");
    easy.difficulty = Some(NumValue::Num(2.0));
    assert_eq!(
        ids_by(vec![hard, easy], "difficulty", &cfg),
        vec!["d-easy", "d-hard"]
    );
}

#[test]
fn sort_by_specs_orders_by_ready_band() {
    use crate::ready::Readiness;
    let cfg = config();
    let mut blocked = task("r-blocked", "wip", 5.0, None);
    blocked.links = vec![dep_edge("missing.typ")];
    let ready = task("r-ready", "wip", 5.0, None);
    let mut tasks = vec![blocked, ready];
    let readiness = Readiness::compute(&tasks, &cfg);
    sort_by_specs(&mut tasks, &["ready".to_owned()], &cfg, &readiness).unwrap();
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["r-ready", "r-blocked"]);
}

#[test]
fn sort_by_specs_orders_by_title() {
    let cfg = config();
    let mut banana = bare_task("t-banana");
    banana.title = "banana".to_owned();
    let mut apple = bare_task("t-apple");
    apple.title = "apple".to_owned();
    assert_eq!(
        ids_by(vec![banana, apple], "title", &cfg),
        vec!["t-apple", "t-banana"]
    );
}

#[test]
fn sort_by_specs_orders_by_kind() {
    let cfg = config();
    let mut chore = bare_task("k-task");
    chore.kind = Some("task".to_owned());
    let mut bug = bare_task("k-bug");
    bug.kind = Some("bug".to_owned());
    assert_eq!(
        ids_by(vec![chore, bug], "kind", &cfg),
        vec!["k-bug", "k-task"]
    );
}

#[test]
fn sort_by_specs_orders_by_due_ascending() {
    let cfg = config();
    let mut late = bare_task("due-late");
    late.due = Some("2026-09-01".to_owned());
    let mut early = bare_task("due-early");
    early.due = Some("2026-07-01".to_owned());
    assert_eq!(
        ids_by(vec![late, early], "due", &cfg),
        vec!["due-early", "due-late"]
    );
}

#[test]
fn sort_by_specs_orders_by_start_ascending() {
    let cfg = config();
    let mut late = bare_task("start-late");
    late.start = Some("2026-09-01".to_owned());
    let mut early = bare_task("start-early");
    early.start = Some("2026-07-01".to_owned());
    assert_eq!(
        ids_by(vec![late, early], "start", &cfg),
        vec!["start-early", "start-late"]
    );
}

#[test]
fn sort_by_specs_orders_by_file_path() {
    let cfg = config();
    // bare_task sets file = tasks/{id}.typ; input reverses file order.
    let tasks = vec![bare_task("f-b"), bare_task("f-a")];
    assert_eq!(ids_by(tasks, "file", &cfg), vec!["f-a", "f-b"]);
}

#[test]
fn sort_by_specs_orders_by_depth() {
    let cfg = config();
    // bare_task sets depth 0; give the others distinct depths in inverse sorted
    // order so a dropped comparator arm (equal keys) fails the assertion.
    let mut deep = bare_task("d-deep");
    deep.depth = 2;
    let mut shallow = bare_task("d-shallow");
    shallow.depth = 1;
    let root = bare_task("d-root");
    assert_eq!(
        ids_by(vec![deep, root, shallow], "depth", &cfg),
        vec!["d-root", "d-shallow", "d-deep"]
    );
}

#[test]
fn sort_by_specs_orders_by_file_path_across_a_mixed_set() {
    let cfg = config();
    let mut root = bare_task("m-root");
    root.file = "zzz.typ".into(); // no folder at all
    let mut nested_a = bare_task("m-nested-a");
    nested_a.file = "tasks/sub/a.typ".into();
    let mut nested_b = bare_task("m-nested-b");
    nested_b.file = "tasks/sub/b.typ".into();
    let mut flat = bare_task("m-flat");
    flat.file = "tasks/top.typ".into();

    // Input order deliberately scrambled; expected order is the full
    // root-relative path compared byte-wise, folder and file both.
    let tasks = vec![
        root.clone(),
        nested_b.clone(),
        flat.clone(),
        nested_a.clone(),
    ];
    assert_eq!(
        ids_by(tasks, "file", &cfg),
        vec!["m-nested-a", "m-nested-b", "m-flat", "m-root"]
    );
}
