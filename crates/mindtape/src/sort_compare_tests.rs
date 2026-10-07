//! Every per-field `--sort` arm routed through `sort_by_specs` — the
//! `compare_field` dispatch exercised end to end. Shares `sort_tests.rs`'s
//! `config()`/`task()` fixtures; the comparators' own unit tests live in
//! `sort_compare_tests_comparators.rs`.
#![allow(clippy::unwrap_used)]

use super::sort_by_specs;
use super::tests::{config, task};
use crate::config::{Config, parse_config};
use crate::model::Task;
use crate::model::numeric::NumValue;
use crate::test_support::{bare_task, dep_edge};

/// Ids after sorting `tasks` by a single `field` spec (no readiness signal).
fn ids_by(mut tasks: Vec<Task>, field: &str, cfg: &Config) -> Vec<String> {
    sort_by_specs(
        &mut tasks,
        &[field.to_owned()],
        cfg,
        &crate::graph::ready::Readiness::empty(),
    )
    .unwrap();
    tasks.iter().map(|t| t.id.clone()).collect()
}

// Per-field `--sort` comparators: each `compare_field` special case and each
// `crate::model::fields` registry entry must route to its own key. Input order is
// the inverse of the expected sorted order, so dropping the special case (or
// the registry entry, which would fail at `sort_by_specs'` own field check
// first) fails the assertion.

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
fn sort_by_specs_orders_by_status_name_alphabetically() {
    // The explicit name property sorts lexicographically, unlike bare `status`
    // (legend order) and `status.index` (the same order numerically).
    let cfg = config();
    let tasks = vec![
        task("s-wip", "wip", 5.0, None),
        task("s-done", "done", 5.0, None),
    ];
    assert_eq!(ids_by(tasks, "status.name", &cfg), vec!["s-done", "s-wip"]);
}

#[test]
fn sort_by_specs_orders_by_status_index_like_the_legend() {
    let cfg = config();
    let tasks = vec![
        task("s-done", "done", 5.0, None),
        task("s-proposed", "proposed", 5.0, None),
    ];
    assert_eq!(
        ids_by(tasks, "status.index", &cfg),
        vec!["s-proposed", "s-done"]
    );
}

#[test]
fn sort_by_specs_orders_by_status_date() {
    let cfg = config();
    let dated = |id: &str, date: &str| {
        let mut task = task(id, "wip", 5.0, None);
        task.status.as_mut().unwrap().date = Some(date.to_owned());
        task
    };
    let tasks = vec![dated("new", "2026-09-01"), dated("old", "2026-01-02")];
    assert_eq!(ids_by(tasks, "status.date", &cfg), vec!["old", "new"]);
}

#[test]
fn sort_by_specs_orders_a_missing_status_date_last() {
    let cfg = config();
    let mut undated = task("undated", "wip", 5.0, None);
    let mut dated = task("dated", "wip", 5.0, None);
    dated.status.as_mut().unwrap().date = Some("2026-01-02".to_owned());
    undated.status.as_mut().unwrap().date = None;
    assert_eq!(
        ids_by(vec![undated, dated], "status.date", &cfg),
        vec!["dated", "undated"]
    );
}

#[test]
fn sort_by_specs_descending_orders_a_missing_status_date_first() {
    // `:desc` reverses the whole comparison, so the missing-last rule of the
    // ascending case becomes missing-first — the documented convention every
    // field follows.
    let cfg = config();
    let mut dated = task("dated", "wip", 5.0, None);
    dated.status.as_mut().unwrap().date = Some("2026-01-02".to_owned());
    let undated = task("undated", "wip", 5.0, None);
    assert_eq!(
        ids_by(vec![dated, undated], "status.date:desc", &cfg),
        vec!["undated", "dated"]
    );
}

#[test]
fn sort_by_specs_orders_by_status_note() {
    let cfg = config();
    let noted = |id: &str, note: &str| {
        let mut task = task(id, "wip", 5.0, None);
        task.status.as_mut().unwrap().note = Some(note.to_owned());
        task
    };
    let tasks = vec![noted("b", "second"), noted("a", "first")];
    assert_eq!(ids_by(tasks, "status.note", &cfg), vec!["a", "b"]);
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

// Priority/difficulty resolve *labels* through the configured bands to a scale
// point, so `--sort` orders by that numeric point. A label's lexicographic
// order can disagree with its numeric order — the wedge that pins the
// `compare_field` arms: `high`(6) sorts after `low`(1) numerically but before
// it as a string, so deleting the arm (falling back to the raw label string
// compare) flips the order.

/// Bands whose labels sort opposite to their numeric order in both fields:
/// `low`/`soft` = 1 and `high`/`hard` = 6 — each "bigger" label is
/// lexicographically the *smaller* string.
fn inverted_label_config() -> Config {
    parse_config(
        r#"
[[tasks.status]]
value = "wip"

[tasks.priority]
min = 1
max = 10
[[tasks.priority.bands]]
label = "low"
min = 1
max = 5
[[tasks.priority.bands]]
label = "high"
min = 6
max = 10

[tasks.difficulty]
min = 1
max = 10
anchors = []
[[tasks.difficulty.bands]]
label = "soft"
min = 1
max = 5
[[tasks.difficulty.bands]]
label = "hard"
min = 6
max = 10
"#,
    )
    .unwrap()
}

#[test]
fn sort_by_specs_orders_priority_labels_numerically() {
    let cfg = inverted_label_config();
    let mut high = bare_task("p-high");
    high.priority = Some(NumValue::Label("high".to_owned()));
    let mut low = bare_task("p-low");
    low.priority = Some(NumValue::Label("low".to_owned()));
    // `high` = 6 > `low` = 1 numerically, but `high` < `low` as strings: the
    // numeric arm sorts low first, a deleted arm would sort high first.
    assert_eq!(
        ids_by(vec![high, low], "priority", &cfg),
        vec!["p-low", "p-high"]
    );
}

#[test]
fn sort_by_specs_orders_difficulty_labels_numerically() {
    let cfg = inverted_label_config();
    let mut hard = bare_task("d-hard");
    hard.difficulty = Some(NumValue::Label("hard".to_owned()));
    let mut soft = bare_task("d-soft");
    soft.difficulty = Some(NumValue::Label("soft".to_owned()));
    // `hard` = 6 > `soft` = 1 numerically, but `hard` < `soft` as strings: the
    // numeric arm sorts soft first, a deleted arm would sort hard first.
    assert_eq!(
        ids_by(vec![hard, soft], "difficulty", &cfg),
        vec!["d-soft", "d-hard"]
    );
}

#[test]
fn sort_by_specs_orders_by_ready_band() {
    use crate::graph::ready::Readiness;
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
    assert_eq!(ids_by(tasks, "path", &cfg), vec!["f-a", "f-b"]);
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
fn sort_by_specs_orders_by_assignee_ascending() {
    let cfg = config();
    let mut zed = bare_task("a-zed");
    zed.assignee = Some("zed".to_owned());
    let mut ada = bare_task("a-ada");
    ada.assignee = Some("ada".to_owned());
    assert_eq!(
        ids_by(vec![zed, ada], "assignee", &cfg),
        vec!["a-ada", "a-zed"]
    );
}

#[test]
fn sort_by_specs_orders_by_reviewed_ascending() {
    let cfg = config();
    let mut late = bare_task("rev-late");
    late.reviewed = Some("2026-09-01".to_owned());
    let mut early = bare_task("rev-early");
    early.reviewed = Some("2026-07-01".to_owned());
    assert_eq!(
        ids_by(vec![late, early], "reviewed", &cfg),
        vec!["rev-early", "rev-late"]
    );
}

#[test]
fn sort_by_specs_sorts_missing_generic_field_last_ascending() {
    // Regression: before routing through the shared `crate::model::fields` registry,
    // a missing optional field (e.g. `kind`) fell back to `""`, which sorts
    // *first* ascending — inconsistent with every numeric field's
    // missing-sorts-last rule. A missing `kind` must trail a present one
    // ascending, and (since `:desc` reverses the whole comparison, same as
    // `priority`/`difficulty`) lead it descending.
    let cfg = config();
    let mut has_kind = bare_task("k-has");
    has_kind.kind = Some("bug".to_owned());
    let no_kind = bare_task("k-none");
    assert_eq!(
        ids_by(vec![no_kind.clone(), has_kind.clone()], "kind", &cfg),
        vec!["k-has", "k-none"]
    );
    let mut tasks = vec![no_kind, has_kind];
    sort_by_specs(
        &mut tasks,
        &["kind:desc".to_owned()],
        &cfg,
        &crate::graph::ready::Readiness::empty(),
    )
    .unwrap();
    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["k-none", "k-has"]);
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
        ids_by(tasks, "path", &cfg),
        vec!["m-nested-a", "m-nested-b", "m-flat", "m-root"]
    );
}
