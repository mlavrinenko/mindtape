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
fn validate_sort_spec_accepts_every_registry_field_and_ready() {
    // The registry is the one field vocabulary: a field or property wired
    // into `crate::fields` is sortable by construction, and `ready` is the
    // one computed rank with no registry entry behind it.
    for def in crate::fields::FIELDS {
        assert!(
            validate_sort_spec(def.name).is_ok(),
            "expected {} to be valid",
            def.name
        );
        assert!(validate_sort_spec(&format!("{}:asc", def.name)).is_ok());
        assert!(validate_sort_spec(&format!("{}:desc", def.name)).is_ok());
    }
    assert!(validate_sort_spec("ready").is_ok());
}

#[test]
fn validate_sort_spec_rejects_the_filter_only_role_field() {
    // `role` resolves through the `[[actors]]` registry at read time and has
    // no value a comparator could order, so it stays out of the sort
    // vocabulary even though it is a filter field.
    assert!(validate_sort_spec("role").is_err());
}

#[test]
fn validate_sort_spec_rejects_unknown_field() {
    assert!(validate_sort_spec("no_such_field").is_err());
    assert!(validate_sort_spec("no_such_field:desc").is_err());
}

/// `--sort file` is the same trap the filter field was: the column is named
/// `file`, the sort key is `path`. Both the pre-flight validator and
/// `sort_by_specs`' own guard must name the replacement, since a saved
/// `[ls] default-sort` reaches one and a `--sort` flag the other.
#[test]
fn the_retired_file_sort_field_errors_naming_path() {
    let spec = validate_sort_spec("file").unwrap_err();
    assert!(spec.contains("`path`"), "{spec}");

    let applied = sort_by_specs(
        &mut [task("a", "wip", 5.0, None)],
        &["file:desc".to_owned()],
        &config(),
        &crate::ready::Readiness::empty(),
    )
    .unwrap_err();
    assert!(applied.contains("`path`"), "{applied}");
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

// --- Graph-derived sort keys (`topo`, `blocks`) ---

/// A task carrying the graph ranks the loader derives (`crate::dep_rank`),
/// plus a priority — the shape the tiebreak tests need.
fn ranked(id: &str, topo: u32, blocks: u32, priority: f64) -> Task {
    let mut t = task(id, "wip", priority, None);
    t.topo = topo;
    t.blocks = blocks;
    t
}

fn ids(tasks: &[Task]) -> Vec<&str> {
    tasks.iter().map(|t| t.id.as_str()).collect()
}

fn sorted(mut tasks: Vec<Task>, specs: &[&str]) -> Vec<Task> {
    let specs: Vec<String> = specs.iter().map(|s| (*s).to_owned()).collect();
    sort_by_specs(
        &mut tasks,
        &specs,
        &config(),
        &crate::ready::Readiness::empty(),
    )
    .unwrap();
    tasks
}

#[test]
fn sort_topo_puts_dependencies_before_dependents() {
    let tasks = sorted(
        vec![
            ranked("dependent", 2, 0, 5.0),
            ranked("root", 0, 2, 5.0),
            ranked("middle", 1, 1, 5.0),
        ],
        &["topo"],
    );
    assert_eq!(ids(&tasks), vec!["root", "middle", "dependent"]);
}

#[test]
fn sort_topo_leaves_a_tie_for_the_next_spec_to_break() {
    // The declared tiebreak: `topo` is a level, not a linear position, so
    // independent tasks tie and the following spec decides — which is what
    // makes `--sort topo --sort priority:desc` read as "graph order, priority
    // within a layer".
    let tasks = sorted(
        vec![
            ranked("cheap", 0, 0, 1.0),
            ranked("late", 1, 0, 9.0),
            ranked("valuable", 0, 0, 9.0),
        ],
        &["topo", "priority:desc"],
    );
    assert_eq!(ids(&tasks), vec!["valuable", "cheap", "late"]);
}

#[test]
fn sort_blocks_desc_floats_the_highest_leverage_task() {
    // The inversion the feature exists to fix: `hub` scores lowest on priority
    // and highest on leverage, and `blocks:desc` is what surfaces it.
    let tasks = sorted(
        vec![
            ranked("isolated", 0, 0, 9.0),
            ranked("hub", 0, 5, 1.0),
            ranked("small", 0, 1, 7.0),
        ],
        &["blocks:desc"],
    );
    assert_eq!(ids(&tasks), vec!["hub", "small", "isolated"]);
}

#[test]
fn sort_blocks_breaks_ties_on_the_next_spec() {
    let tasks = sorted(
        vec![
            ranked("low", 0, 3, 2.0),
            ranked("high", 0, 3, 8.0),
            ranked("lonely", 0, 0, 9.0),
        ],
        &["blocks:desc", "priority:desc"],
    );
    assert_eq!(ids(&tasks), vec!["high", "low", "lonely"]);
}

#[test]
fn topo_and_blocks_are_accepted_sort_specs() {
    assert!(validate_sort_spec("topo").is_ok());
    assert!(validate_sort_spec("blocks:desc").is_ok());
}
