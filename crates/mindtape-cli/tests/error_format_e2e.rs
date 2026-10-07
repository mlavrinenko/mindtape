//! The error format, end to end: every stderr line starts with `error:`,
//! `hint:`, `note:` or `warning:`, a value the user typed is single-quoted, a
//! single target gets no batch summary, and the exit code says whether the
//! command line was wrong (2) or the work failed (1).
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

mod support;

/// `mt <args>` in `dir`: the exit code and stderr.
fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let output = support::mt(dir).args(args).output().unwrap();
    (
        output.status.code(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn two_tasks() -> tempfile::TempDir {
    support::project_with_tasks(&[("alpha.typ", "Alpha"), ("beta.typ", "Beta")])
}

/// Each line starts with one of the four prefixes, and none is Rust's own.
fn assert_prefixed(stderr: &str) {
    for line in stderr.lines() {
        assert!(
            ["error: ", "hint: ", "note: ", "warning: "]
                .iter()
                .any(|prefix| line.starts_with(prefix)),
            "unprefixed line {line:?} in:\n{stderr}"
        );
    }
}

#[test]
fn an_unknown_task_is_one_error_and_a_hint_with_no_summary() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["wip", "nonexistent-xyz"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        stderr,
        "error: no task: 'nonexistent-xyz'\nhint: run mt ls to see your tasks\n"
    );
}

#[test]
fn a_near_miss_names_the_task_it_meant() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["wip", "alpah"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        stderr,
        "error: no task: 'alpah'\nhint: did you mean alpha?\n"
    );
}

#[test]
fn a_batch_with_a_failure_ends_with_one_summary() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["wip", "alpha", "nonexistent-xyz"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_prefixed(&stderr);
    assert!(
        stderr.contains("error: no task: 'nonexistent-xyz'\n"),
        "{stderr}"
    );
    assert!(
        stderr.ends_with("error: flipped 1 of 2 tasks; 1 failed\n"),
        "{stderr}"
    );
}

#[test]
fn a_note_passed_as_a_task_gets_a_hint_to_use_m() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["done", "alpha", "it's fixed"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("error: no task: 'it'\\''s fixed'\n"),
        "{stderr}"
    );
    assert!(
        stderr
            .contains("hint: to add a note, pass it with -m: mt done alpha -m 'it'\\''s fixed'\n"),
        "{stderr}"
    );
}

#[test]
fn an_unknown_field_in_a_filter_is_a_usage_error() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "status == wip"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown field: 'wip' in filter 'status == wip'\n"
    );
}

#[test]
fn a_filter_that_does_not_parse_is_a_usage_error_on_one_line() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "foo("]);

    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.starts_with("error: can't parse filter: 'foo(': "),
        "{stderr}"
    );
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(!stderr.trim_end().ends_with('.'), "{stderr}");
}

#[test]
fn an_unknown_command_is_a_usage_error() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["dne", "alpha"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown command: 'dne'\nhint: did you mean done?\n"
    );
}

#[test]
fn an_unknown_status_is_a_usage_error() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["flip", "nostatus", "alpha"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.starts_with("error: unknown status: 'nostatus'\nhint: "),
        "{stderr}"
    );
    assert_prefixed(&stderr);
}

/// An error no call site has converted yet still comes out as one prefixed
/// line, its causes joined with `: `.
#[test]
fn an_unconverted_error_chain_is_one_line_without_caused_by() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["add", "-f", "/nonexistent", "Gamma"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_prefixed(&stderr);
    assert!(!stderr.contains("Caused by"), "{stderr}");
    assert!(
        stderr.starts_with("error: read values from /nonexistent: "),
        "{stderr}"
    );
}

#[test]
fn an_invalid_pinned_date_names_the_value() {
    let dir = two_tasks();

    let output = support::mt(dir.path())
        .env("MINDTAPE_NOW", "garbage")
        .args(["ls"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert_eq!(
        stderr,
        "error: MINDTAPE_NOW is not a date: 'garbage'\n\
         hint: set it to YYYY-MM-DD or an RFC 3339 timestamp, or unset it\n"
    );
}

#[test]
fn colour_goes_on_the_prefix_only() {
    let dir = two_tasks();

    let (_, stderr) = run(dir.path(), &["--color", "always", "wip", "nonexistent-xyz"]);

    assert!(
        stderr.starts_with("\u{1b}[1m\u{1b}[31merror:\u{1b}[0m no task: 'nonexistent-xyz'\n"),
        "{stderr:?}"
    );
}
