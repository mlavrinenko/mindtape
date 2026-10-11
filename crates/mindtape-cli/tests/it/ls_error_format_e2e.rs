//! `mt ls` problems in the error format: the value single-quoted, the next
//! step on a `hint:` line, and exit 2 for a wrong command line.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

use support::run_stderr as run;

fn project() -> tempfile::TempDir {
    support::project_with_tasks(&[("alpha.typ", "Alpha")])
}

#[test]
fn an_unknown_sort_field_names_the_nearest_field() {
    let (code, stderr) = run(project().path(), &["ls", "--sort", "priorty"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown sort field: 'priorty'\nhint: did you mean priority?\n"
    );
}

#[test]
fn an_unknown_sort_direction_says_the_two_choices() {
    let (code, stderr) = run(project().path(), &["ls", "--sort", "priority:up"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown sort direction: 'up'\nhint: use asc or desc\n"
    );
}

#[test]
fn interactive_on_a_pipe_is_a_usage_error_with_a_hint() {
    let (code, stderr) = run(project().path(), &["ls", "-i"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: --interactive needs a terminal on stdin and stdout\nhint: drop --interactive when piping\n"
    );
}

#[test]
fn interactive_with_json_is_a_usage_error_with_a_hint() {
    let (code, stderr) = run(project().path(), &["ls", "-i", "--format", "json"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: --interactive works only with the table format\nhint: drop --format, or drop --interactive\n"
    );
}

#[test]
fn an_unknown_column_names_the_nearest_column() {
    let (code, stderr) = run(project().path(), &["ls", "--columns", "titel"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown column: 'titel'\nhint: did you mean title?\n"
    );
}
