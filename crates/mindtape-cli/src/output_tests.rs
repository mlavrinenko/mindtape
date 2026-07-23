#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use std::collections::BTreeMap;

use mindtape::numeric::NumValue;
use mindtape::test_support::bare_task;
use mindtape::{Status, Task};
use typst_harvest::HVal;

use mindtape::{Config, Readiness};

use super::{Applied, OutputFormat, render, task_to_json};

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
    // Populated so `due` stays significant under the default `auto`
    // column visibility — several csv tests below assert on its presence.
    t.due = Some("2026-07-01".to_owned());
    t
}

#[test]
fn table_contains_fields() {
    let out = render(
        &[sample()],
        OutputFormat::Table,
        &Config::default(),
        &Readiness::empty(),
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
        &Config::default(),
        &Readiness::empty(),
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
        &Config::default(),
        &Readiness::empty(),
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
        &Config::default(),
        &Readiness::empty(),
        None,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value[0]["id"], "t1");
    assert_eq!(value[0]["status"]["name"], "wip");
    assert_eq!(value[0]["estimate"]["hours"], 8.0);
}

/// Render one `sample()` task as `--format json`, sharing the render
/// boilerplate across the tests below (jscpd flags it as a clone once
/// duplicated three times over).
fn json_out(applied: Option<&Applied>) -> String {
    render(
        &[sample()],
        OutputFormat::Json,
        &Config::default(),
        &Readiness::empty(),
        applied,
    )
    .unwrap()
}

/// [`json_out`], parsed back into a [`serde_json::Value`].
fn json_value(applied: Option<&Applied>) -> serde_json::Value {
    serde_json::from_str(&json_out(applied)).unwrap()
}

#[test]
fn json_with_applied_wraps_tasks_in_envelope() {
    // `mt ls --json` (an `applied` present) is the breaking envelope
    // shape (`truth/tasks/conditional-ls-rules.typ`); `mt eval --json`
    // (no `applied`) stays a bare array, as covered above.
    let applied = Applied {
        rule: Some("release".to_owned()),
        filter_from: "inbox".to_owned(),
        filter_value: "priority > 5".to_owned(),
        sort_from: "release".to_owned(),
        sort_value: vec!["priority:desc".to_owned()],
        columns_from: "inbox".to_owned(),
        columns_value: vec!["status".to_owned(), "title".to_owned()],
    };
    let value = json_value(Some(&applied));
    assert_eq!(value["tasks"][0]["id"], "t1");
    assert_eq!(value["applied"]["rule"], "release");
    assert_eq!(value["applied"]["facets"]["filter"]["from"], "inbox");
    assert_eq!(
        value["applied"]["facets"]["filter"]["value"],
        "priority > 5"
    );
    assert_eq!(value["applied"]["facets"]["sort"]["from"], "release");
    assert_eq!(
        value["applied"]["facets"]["sort"]["value"][0],
        "priority:desc"
    );
    assert_eq!(value["applied"]["facets"]["columns"]["from"], "inbox");
}

#[test]
fn json_with_applied_carries_schema_version() {
    // The envelope is document-level, so `schema_version` sits alongside
    // `tasks`/`applied`, not on each task record
    // (`truth/tasks/emit-schema-version-in-the-ls-json-envelope.typ`).
    let value = json_value(Some(&Applied::default()));
    assert_eq!(value["schema_version"], mindtape::SCHEMA_VERSION);
    assert!(value["tasks"][0]["schema_version"].is_null());
}

#[test]
fn json_without_applied_stays_a_bare_array_with_no_schema_version() {
    let out = json_out(None);
    assert!(!out.contains("schema_version"));
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(value.is_array());
}

#[test]
fn json_with_no_rule_matched_reports_null_rule() {
    let applied = Applied {
        rule: None,
        filter_from: "match-all".to_owned(),
        ..Applied::default()
    };
    let value = json_value(Some(&applied));
    assert!(value["applied"]["rule"].is_null());
    assert_eq!(value["applied"]["facets"]["filter"]["from"], "match-all");
}

#[test]
fn csv_has_header_and_row() {
    let out = render(
        &[sample()],
        OutputFormat::Csv,
        &Config::default(),
        &Readiness::empty(),
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
    let mut config = Config::default();
    config.ls.columns_order = vec!["title".to_owned(), "status".to_owned()];
    let out = render(
        &[sample()],
        OutputFormat::Csv,
        &config,
        &Readiness::empty(),
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
        &Config::default(),
        &readiness,
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
    let out = render(
        &[task],
        OutputFormat::Csv,
        &Config::default(),
        &Readiness::empty(),
        None,
    )
    .unwrap();
    assert_eq!(out.lines().next().unwrap(), "id,status,priority,due,title");
    assert!(
        !out.contains("bug-report"),
        "kind has no ls column and must not leak into csv"
    );
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
