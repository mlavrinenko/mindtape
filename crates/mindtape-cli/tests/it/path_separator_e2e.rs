//! `--path-separator` sets the separator of every path `mt` shows to people
//! and leaves alone what a program reads. A Linux run passes `\`, so it pins
//! the route a Windows run takes by default.
#![allow(clippy::unwrap_used)]

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
