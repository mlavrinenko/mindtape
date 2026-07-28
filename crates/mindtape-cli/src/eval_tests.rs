#![allow(clippy::unwrap_used)]

use super::{EvalArgs, run};

const TASK: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

#[test]
fn eval_valid_file_table_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, TASK).unwrap();
    assert!(run(&EvalArgs { file, json: false }).is_ok());
}

#[test]
fn eval_valid_file_json_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, TASK).unwrap();
    assert!(run(&EvalArgs { file, json: true }).is_ok());
}

#[test]
fn eval_missing_file_returns_err() {
    // A path under a fresh tempdir that was never written: nonexistent by
    // construction on every platform, unlike a hardcoded `/tmp/...`.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("no_such_file.typ");
    assert!(run(&EvalArgs { file, json: false }).is_err());
}

#[test]
fn eval_file_without_marker_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.typ");
    std::fs::write(&file, "= Just notes\n\nNo marker here.\n").unwrap();
    assert!(run(&EvalArgs { file, json: false }).is_err());
}

#[test]
fn build_world_missing_file_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("no_such_file.typ");
    assert!(super::build_world(&file).is_err());
}
