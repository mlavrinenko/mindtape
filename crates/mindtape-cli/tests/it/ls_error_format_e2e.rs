//! `mt ls` problems in the error format: the value single-quoted, the next
//! step on a `hint:` line, and exit 2 for a wrong command line.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

fn project() -> tempfile::TempDir {
    support::project_with_tasks(&[("alpha.typ", "Alpha")])
}

#[test]
fn an_unknown_sort_field_names_the_nearest_field() {
    support::assert_stderr(
        project().path(),
        &["ls", "--sort", "priorty"],
        2,
        "error: unknown sort field: 'priorty'\nhint: did you mean priority?\n",
    );
}

#[test]
fn an_unknown_sort_direction_says_the_two_choices() {
    support::assert_stderr(
        project().path(),
        &["ls", "--sort", "priority:up"],
        2,
        "error: unknown sort direction: 'up'\nhint: use asc or desc\n",
    );
}

#[test]
fn interactive_on_a_pipe_is_a_usage_error_with_a_hint() {
    support::assert_stderr(
        project().path(),
        &["ls", "-i"],
        2,
        "error: --interactive needs a terminal on stdin and stdout\nhint: drop --interactive when piping\n",
    );
}

#[test]
fn interactive_with_json_is_a_usage_error_with_a_hint() {
    support::assert_stderr(
        project().path(),
        &["ls", "-i", "--format", "json"],
        2,
        "error: --interactive works only with the table format\nhint: drop --format, or drop --interactive\n",
    );
}

#[test]
fn an_unknown_column_names_the_nearest_column() {
    support::assert_stderr(
        project().path(),
        &["ls", "--columns", "titel"],
        2,
        "error: unknown column: 'titel'\nhint: did you mean title?\n",
    );
}
