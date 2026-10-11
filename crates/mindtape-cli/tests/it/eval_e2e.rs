//! End-to-end tests for `mt eval` — the one command that renders a task
//! against no project at all.
//!
//! That makes it the only surface where the config it renders against is a
//! choice rather than a lookup, and since every column's visibility now lives
//! in `crates/mindtape/src/config/defaults.toml` as a `when` expression, the choice
//! decides whether anything renders: `Config::default()` bypasses the floor, so
//! it describes no column and prints a header-less blank
//! (`truth/tasks/collapse-column-visibility-onto-a-single-when-expression.typ`).
//! `mt eval` reads the floor instead, and these pin that from the outside —
//! stdout, not a resolved column list.
#![allow(clippy::unwrap_used)]

use crate::support;

use support::stdout;

#[test]
fn eval_renders_the_task_as_a_table() {
    let dir = support::project_with_task("Evaluate me");
    let text = stdout(dir.path(), &["eval", "tasks/test.typ"]);

    assert!(
        text.contains("TITLE") && text.contains("Evaluate me"),
        "eval must print the table it resolved, header and row: {text}"
    );
    // A column whose `when` only the embedded floor supplies. Rendering it is
    // what tells the floor apart from the empty config — the latter would
    // print nothing at all, header included.
    assert!(
        text.contains("STATUS") && text.contains("wip"),
        "eval renders against the floor, so a populated column shows: {text}"
    );
}

#[test]
fn eval_json_emits_the_task_object() {
    let dir = support::project_with_task("Evaluate me");
    let text = stdout(dir.path(), &["eval", "tasks/test.typ", "--json"]);
    let task: serde_json::Value = serde_json::from_str(&text).unwrap();

    // A bare object, not the `ls` envelope: one file is not a listing, so there
    // is nothing for `applied` or `truncated` to describe.
    assert_eq!(support::at(&task, "/title").as_str(), Some("Evaluate me"));
    assert_eq!(support::at(&task, "/id").as_str(), Some("test"));
    assert!(
        task.get("tasks").is_none(),
        "eval --json is the task itself, not an envelope around a list: {text}"
    );
}

#[test]
fn eval_resolves_a_stem_like_the_other_task_commands() {
    let dir = support::project_with_task("Evaluate me");
    let text = stdout(dir.path(), &["eval", "test", "--json"]);
    let task: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert_eq!(support::at(&task, "/title").as_str(), Some("Evaluate me"));
}

#[test]
fn eval_names_the_nearest_stem_on_an_unknown_reference() {
    let dir = support::project_with_task("Evaluate me");
    let out = support::mt(dir.path())
        .args(["eval", "tset"])
        .output()
        .unwrap();

    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("hint: did you mean test?"), "{err}");
}
