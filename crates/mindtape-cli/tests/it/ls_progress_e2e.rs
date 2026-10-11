//! End-to-end tests for the computed `children` and `progress` fields: how far
//! a parent's children have got, read off one listing instead of walking the
//! `parent` edges.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::support;

use support::stdout;

/// `epic` with three children, one of them done.
fn epic_project() -> tempfile::TempDir {
    let dir = support::project_with_tasks(&[
        ("epic.typ", "Epic"),
        ("a.typ", "A"),
        ("b.typ", "B"),
        ("c.typ", "C"),
    ]);
    for child in ["a", "b", "c"] {
        support::run_ok(dir.path(), &["link", child, "parent", "epic"]);
    }
    support::run_ok(dir.path(), &["done", "a"]);
    dir
}

#[test]
fn the_columns_render_the_count_and_done_over_children() {
    let dir = epic_project();
    let text = stdout(
        dir.path(),
        &["ls", "children > 0", "--columns", "title,children,progress"],
    );

    assert!(text.contains("CHILDREN"), "{text}");
    assert!(text.contains("PROGRESS"), "{text}");
    assert!(text.contains("1/3"), "{text}");
    assert!(!text.contains("B"), "only the parent matches:\n{text}");
}

#[test]
fn progress_filters_and_sorts_as_a_fraction() {
    let dir = epic_project();
    let text = stdout(dir.path(), &["ls", "progress < 0.5", "--format", "path"]);
    assert_eq!(text.trim(), "tasks/epic.typ");
    let none = stdout(dir.path(), &["ls", "progress == 1", "--format", "path"]);
    assert_eq!(none.trim(), "");
    stdout(dir.path(), &["ls", "", "--sort", "progress:desc"]);
}

#[test]
fn json_carries_the_count_and_the_fraction() {
    let dir = epic_project();
    let epic = support::task_json(dir.path(), "epic");
    assert_eq!(epic["children"], 3);
    assert!((epic["progress"].as_f64().unwrap() - 1.0 / 3.0).abs() < 1e-9);
    let leaf = support::task_json(dir.path(), "b");
    assert_eq!(leaf["children"], 0);
    assert!(leaf["progress"].is_null());
}

#[test]
fn csv_carries_the_raw_count_and_fraction() {
    let dir = epic_project();
    let text = stdout(
        dir.path(),
        &[
            "ls",
            "children > 0",
            "--columns",
            "children,progress",
            "--format",
            "csv",
        ],
    );
    assert!(text.contains("children,progress"), "{text}");
    assert!(text.contains("3,0.333"), "{text}");
}
