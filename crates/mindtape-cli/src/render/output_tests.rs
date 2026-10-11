#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use std::collections::BTreeMap;

use mindtape::model::numeric::NumValue;
use mindtape::test_support::{bare_task, floor_config};
use mindtape::{Status, Task};
use typst_harvest::HVal;

use mindtape::{Config, Readiness};

use crate::render::TestCtx;

use super::{OutputFormat, render, task_to_json};

fn sample() -> Task {
    let mut t = bare_task("t1");
    t.title = "Build".to_owned();
    t.status = Some(Status {
        name: "wip".to_owned(),
        date: Some("2026-06-14".to_owned()),
        note: None,
    });
    t.priority = Some(NumValue::Num(8.0));
    t.estimate = BTreeMap::from([("hours".to_owned(), 8.0)]);
    t.tags = vec!["x".to_owned()];
    // Populated so `due` clears its floored `nonempty()` — several csv tests
    // below assert on its presence.
    t.due = Some("2026-07-01".to_owned());
    t
}

#[test]
fn table_contains_fields() {
    let out = render(
        &[sample()],
        OutputFormat::Table,
        TestCtx::default().ctx(),
        None,
    )
    .unwrap();
    assert!(out.contains("Build"));
    assert!(out.contains("wip"));
}

#[test]
fn table_has_handle_column() {
    let out = render(
        &[sample()],
        OutputFormat::Table,
        TestCtx::default().ctx(),
        None,
    )
    .unwrap();
    // The header row must have "#" and the first task is ordinal 1.
    assert!(out.contains('#'));
    assert!(out.contains("1"));
}

#[test]
fn json_does_not_contain_handle() {
    let out = render(
        &[sample()],
        OutputFormat::Json,
        TestCtx::default().ctx(),
        None,
    )
    .unwrap();
    assert!(
        !out.contains("\"handle\""),
        "handle must not appear in JSON"
    );
    // id is still present
    assert!(out.contains("\"id\""));
}

#[test]
fn json_exposes_core_fields() {
    let out = render(
        &[sample()],
        OutputFormat::Json,
        TestCtx::default().ctx(),
        None,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value[0]["id"], "t1");
    assert_eq!(value[0]["status"]["name"], "wip");
    assert_eq!(value[0]["estimate"]["hours"], 8.0);
}

#[test]
fn csv_has_header_and_row() {
    let out = render(
        &[sample()],
        OutputFormat::Csv,
        TestCtx::default().ctx(),
        None,
    )
    .unwrap();
    assert!(out.contains("id,status,priority,due,title"));
    assert!(out.contains("t1,wip,8,2026-07-01,Build"));
}

#[test]
fn csv_columns_order_selects_and_orders() {
    // A custom `[ls] columns-order` must drive CSV exactly like the table
    // renderer: only the listed columns, in the given sequence.
    let mut config = floor_config();
    config.ls.columns_order = vec!["title".to_owned(), "status".to_owned()];
    let out = render(
        &[sample()],
        OutputFormat::Csv,
        TestCtx::with_config(config).ctx(),
        None,
    )
    .unwrap();
    let mut lines = out.lines();
    assert_eq!(lines.next().unwrap(), "title,status");
    assert_eq!(lines.next().unwrap(), "Build,wip");
}

#[test]
fn csv_includes_ready_when_blocked() {
    // `ready` is one of `LS_COLUMNS`: like the table, CSV must surface it
    // once a task is contested, not just the historical fixed 5 columns.
    let mut blocked = sample();
    blocked.id = "t2".to_owned();
    blocked.file = "tasks/t2.typ".into();
    blocked.links = vec![mindtape::test_support::dep_edge("missing.typ")];
    let tasks = vec![sample(), blocked];
    let readiness = Readiness::compute(&tasks, &Config::default());
    let out = render(
        &tasks,
        OutputFormat::Csv,
        TestCtx::with_readiness(readiness).ctx(),
        None,
    )
    .unwrap();
    assert_eq!(
        out.lines().next().unwrap(),
        "id,status,ready,priority,due,title"
    );
    assert!(out.contains("t2,wip,blocked,8,2026-07-01,Build"));
}

#[test]
fn csv_omits_fields_without_ls_column() {
    // `kind`, `estimate`, `tags`, and `x-<ns>` extras have no `LS_COLUMNS`
    // entry, so — like the table — CSV does not represent them; only
    // `--format json` exposes every field. This pins the documented
    // limitation from `render_csv`'s doc comment as a regression guard.
    let mut task = sample();
    task.kind = Some("bug-report".to_owned());
    let out = render(&[task], OutputFormat::Csv, TestCtx::default().ctx(), None).unwrap();
    assert_eq!(out.lines().next().unwrap(), "id,status,priority,due,title");
    assert!(
        !out.contains("bug-report"),
        "kind has no ls column and must not leak into csv"
    );
}

#[test]
fn csv_file_column_hidden_unless_named() {
    // Bare `mt ls --format csv` (no explicit `[ls] columns-order`) must not
    // surface `file`: its cell is never blank, so the ordinary auto rule
    // cannot apply — significance comes only from being named explicitly.
    let out = render(
        &[sample()],
        OutputFormat::Csv,
        TestCtx::default().ctx(),
        None,
    )
    .unwrap();
    assert!(!out.lines().next().unwrap().contains("file"));
}

#[test]
fn csv_file_column_shown_and_root_relative_when_named() {
    // CSV reads `task.file` directly rather than the table's cwd-relative
    // cell, so this holds for any working directory — what a subdirectory run
    // proves, and this binary cannot, is in `tests/it/ls_path_e2e.rs`
    // (`csv_and_json_report_the_same_root_relative_file_from_a_subdirectory`).
    let mut config = floor_config();
    config.ls.columns_order = vec!["title".to_owned(), "file".to_owned()];
    let mut task = sample();
    task.file = "truth/tasks/t1.typ".into();
    let out = render(
        &[task],
        OutputFormat::Csv,
        TestCtx::with_config(config).ctx(),
        None,
    )
    .unwrap();
    let mut lines = out.lines();
    assert_eq!(lines.next().unwrap(), "title,file");
    assert_eq!(lines.next().unwrap(), "Build,truth/tasks/t1.typ");
}

#[test]
fn path_format_emits_one_path_per_line_no_header() {
    let mut a = sample();
    a.file = "tasks/a.typ".into();
    let mut b = bare_task("t2");
    b.file = "tasks/b.typ".into();
    let out = render(&[a, b], OutputFormat::Path, TestCtx::default().ctx(), None).unwrap();
    assert_eq!(out, "tasks/a.typ\ntasks/b.typ");
}

#[test]
fn path_format_is_empty_for_no_tasks() {
    let out = render(&[], OutputFormat::Path, TestCtx::default().ctx(), None).unwrap();
    assert_eq!(out, "");
}

/// `--format path` is paired with the *table's* `file` column — both read
/// `file_path::cell_text`, so the list you paste into the next command spells
/// each path the way the listing you read it from did. CSV is not part of
/// that pair: it is a record format, and pins the pairing to the wrong side
/// (`truth/tasks/csv-and-json-disagree-about-what-the-file-field-means.typ`).
#[test]
fn path_format_matches_the_table_file_column_cell() {
    let mut task = sample();
    task.file = "truth/tasks/t1.typ".into();

    let cell = crate::render::columns::ColumnId::File.cell(
        "1a",
        &task,
        &Config::default(),
        &Readiness::empty(),
    );

    let path = render(&[task], OutputFormat::Path, TestCtx::default().ctx(), None).unwrap();
    assert_eq!(path, cell.text);
}

/// One value under `file` across both machine formats: CSV's cell is the
/// root-relative path `--format json` carries — the value `mt import` reads
/// back and `contrib/jq/*.jq` select on — never the table's cwd-relative
/// display.
#[test]
fn csv_file_cell_equals_the_json_file_key() {
    let mut config = floor_config();
    config.ls.columns_order = vec!["file".to_owned()];
    let mut task = sample();
    task.file = "truth/tasks/t1.typ".into();

    let csv = render(
        &[task.clone()],
        OutputFormat::Csv,
        TestCtx::with_config(config).ctx(),
        None,
    )
    .unwrap();
    let csv_cell = csv.lines().nth(1).unwrap();

    let json = task_to_json(&task, &Config::default());
    assert_eq!(csv_cell, json["file"].as_str().unwrap());
}

#[test]
fn path0_format_terminates_every_entry_including_the_last() {
    let mut a = sample();
    a.file = "tasks/a.typ".into();
    let mut b = bare_task("t2");
    b.file = "tasks/b.typ".into();
    let out = render(&[a, b], OutputFormat::Path0, TestCtx::default().ctx(), None).unwrap();
    assert_eq!(out, "tasks/a.typ\0tasks/b.typ\0");
    assert_eq!(out.matches('\0').count(), 2, "one NUL per task, incl. last");
}

#[test]
fn path0_format_is_empty_for_no_tasks() {
    let out = render(&[], OutputFormat::Path0, TestCtx::default().ctx(), None).unwrap();
    assert_eq!(out, "");
}

#[test]
fn extras_arrays_and_dicts_serialize() {
    let mut task = sample();
    task.extras.insert(
        "x-ns".to_owned(),
        HVal::Array(vec![HVal::Int(1), HVal::Str("a".to_owned())]),
    );
    let json = task_to_json(&task, &Config::default());
    assert_eq!(json["extras"]["x-ns"][0], 1);
    assert_eq!(json["extras"]["x-ns"][1], "a");
}

#[test]
fn json_exposes_links_as_task_kind_note() {
    let mut task = sample();
    task.links = vec![mindtape::test_support::noted_edge(
        "epic.typ", "related", "why",
    )];
    let json = task_to_json(&task, &Config::default());
    assert_eq!(json["links"][0]["task"], "epic.typ");
    assert_eq!(json["links"][0]["kind"], "related");
    assert_eq!(json["links"][0]["note"], "why");
}

#[test]
fn csv_assignee_column_appears_on_a_mixed_listing() {
    // `assignee` renders off the same `resolve_order` the table uses, so the
    // hidden case is already pinned by every other csv test here (`sample()`
    // carries no assignee, and each asserts the exact header line). What is
    // only reachable through CSV is the raw cell: the actor name verbatim,
    // not the table's styled `CellSpec`.
    let mut a = sample();
    a.assignee = Some("agent".to_owned());
    let mut b = sample();
    b.id = "t2".to_owned();
    b.file = "tasks/t2.typ".into();
    b.assignee = Some("max".to_owned());
    let out = render(&[a, b], OutputFormat::Csv, TestCtx::default().ctx(), None).unwrap();
    let mut lines = out.lines();
    assert_eq!(
        lines.next().unwrap(),
        "id,status,priority,due,assignee,title"
    );
    assert_eq!(lines.next().unwrap(), "t1,wip,8,2026-07-01,agent,Build");
    assert_eq!(lines.next().unwrap(), "t2,wip,8,2026-07-01,max,Build");
}
