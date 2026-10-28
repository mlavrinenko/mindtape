//! `--path-separator` sets the separator of every path `mt` shows to people
//! and leaves alone what a program reads. A Linux run passes `\`, so it pins
//! the route a Windows run takes by default.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

/// What `mt --path-separator \ ls '' <args>` prints over two tasks.
fn listed(args: &[&str]) -> String {
    let dir = support::project_with_tasks(&[("a.typ", "Alpha"), ("b.typ", "Beta")]);
    let mut all = vec!["--path-separator", "\\", "ls", "", "--sort", "path"];
    all.extend_from_slice(args);
    support::stdout(dir.path(), &all)
}

#[test]
fn the_path_format_and_the_file_column_take_the_separator() {
    assert_eq!(
        listed(&["--format", "path"]),
        "tasks\\a.typ\ntasks\\b.typ\n"
    );
    let table = listed(&["--columns", "file"]);
    assert!(table.contains("\ntasks\\a.typ"), "{table}");
}

#[test]
fn the_json_and_csv_file_field_keeps_its_slash() {
    let csv = listed(&["--format", "csv", "--columns", "file"]);
    assert_eq!(csv, "file\ntasks/a.typ\ntasks/b.typ\n");
    let json = listed(&["--format", "json"]);
    assert!(json.contains("\"file\": \"tasks/a.typ\""), "{json}");
}

#[test]
fn a_separator_longer_than_one_character_is_refused() {
    let dir = support::project_with_tasks(&[]);
    let (code, stderr) = support::run_stderr(dir.path(), &["--path-separator", "ab", "ls"]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(stderr.contains("one ASCII character"), "{stderr}");
}

/// A project whose `tasks/a.typ` links to a task that doesn't exist.
fn dangling() -> tempfile::TempDir {
    let dir = support::project_with_tasks(&[("b.typ", "Beta")]);
    let task = mindtape::typst_task!(
        "(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n  \
         links: depends-on(\"nope.typ\"),\n)\n"
    );
    std::fs::write(dir.path().join("tasks/a.typ"), task).unwrap();
    dir
}

#[test]
fn an_error_place_and_the_paths_in_its_message_take_the_separator() {
    let dir = dangling();
    let (code, stderr) = support::run_stderr(dir.path(), &["--path-separator", "\\", "check"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("error: tasks\\a.typ:6: depends-on: link target `tasks\\nope.typ`"),
        "{stderr}"
    );
}

#[test]
fn check_json_keeps_the_slash_in_its_file_and_place() {
    let dir = dangling();
    let out = support::mt(dir.path())
        .args(["--path-separator", "\\", "check", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let problem = support::at(&json, "/problems/0");
    assert_eq!(support::at(problem, "/file"), "tasks/a.typ", "{json}");
    assert_eq!(support::at(problem, "/place"), "tasks/a.typ:6", "{json}");
}

#[test]
fn a_dry_run_plan_takes_the_separator() {
    let dir = support::project_with_tasks(&[("a.typ", "Alpha")]);
    let text = support::stdout(
        dir.path(),
        &["--path-separator", "\\", "mv", "a", "c", "--dry-run"],
    );
    assert!(
        text.contains("would move tasks\\a.typ -> tasks\\c.typ"),
        "{text}"
    );
}
