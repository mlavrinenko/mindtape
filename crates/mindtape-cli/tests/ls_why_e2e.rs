//! End-to-end tests for the computed `why` column and `--json` field: what a
//! blocked task waits on, read off one listing instead of walking its links.
#![allow(clippy::unwrap_used)]

mod support;

use support::stdout;

fn blocked_project() -> tempfile::TempDir {
    let dir = support::project_with_tasks(&[("dep.typ", "Dep"), ("a.typ", "A")]);
    support::run_ok(dir.path(), &["link", "a", "depends-on", "dep"]);
    dir
}

#[test]
fn the_why_column_names_each_unfinished_blocker_with_its_status() {
    let dir = blocked_project();
    let text = stdout(dir.path(), &["ls", "blocked", "--columns", "title,why"]);

    assert!(text.contains("WHY"), "{text}");
    assert!(text.contains("waits on: dep (wip)"), "{text}");
}

#[test]
fn json_carries_why_on_every_task_empty_when_ready() {
    let dir = blocked_project();
    let text = stdout(dir.path(), &["ls", "", "--json"]);
    let listing: serde_json::Value = serde_json::from_str(&text).unwrap();
    let why = |id: &str| {
        support::at(&listing, "/tasks")
            .as_array()
            .unwrap()
            .iter()
            .find(|task| task["id"] == id)
            .unwrap()["why"]
            .clone()
    };

    assert_eq!(
        why("a"),
        serde_json::json!([{"stem": "dep", "status": "wip", "kind": "depends-on"}])
    );
    assert_eq!(why("dep"), serde_json::json!([]));
}
