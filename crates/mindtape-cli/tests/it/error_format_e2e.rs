//! The error format, end to end: every stderr line starts with `error:`,
//! `hint:`, `note:` or `warning:`, a value the user typed is single-quoted, a
//! single target gets no batch summary, and the exit code says whether the
//! command line was wrong (2) or the work failed (1).
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

use support::run_stderr as run;

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
    assert_eq!(
        stderr,
        "error: no task: 'it'\\''s fixed'\n\
         hint: to add a note, pass it with -m: mt done alpha -m 'it'\\''s fixed'\n\
         error: flipped 1 of 2 tasks; 1 failed\n",
        "the -m hint replaces the mt ls hint"
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
fn an_unclosed_parenthesis_is_said_in_plain_words() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "foo("]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: can't parse filter: 'foo(': a '(' is never closed\n"
    );
}

#[test]
fn a_tilde_points_at_matches() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "title ~ \"x\""]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: invalid filter: 'title ~ \"x\"': '~' isn't an operator\n\
         hint: for example: matches(title, \"x\")\n"
    );
}

#[test]
fn a_bare_value_shows_no_rust_type() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "\"wip\""]);

    assert_eq!(code, Some(2), "{stderr}");
    assert!(!stderr.contains("Const"), "{stderr}");
    assert!(!stderr.contains("String("), "{stderr}");
}

#[test]
fn a_call_with_the_wrong_arguments_hints_a_correct_one() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "matches(title)"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: invalid filter: 'matches(title)': matches() takes two arguments\n\
         hint: for example: matches(title, \"^Add \")\n"
    );
}

#[test]
fn a_call_in_place_of_a_field_is_named_as_written() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "word(title, 0) == \"a\""]);

    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.ends_with(": expected a field, got 'word(...)'\n"),
        "{stderr}"
    );
}

#[test]
fn an_unknown_function_suggests_the_nearest() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "matchs(title, \"a\")"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown function: 'matchs' in filter 'matchs(title, \"a\")'\n\
         hint: did you mean matches()?\n"
    );
}

#[test]
fn an_unknown_status_in_a_filter_is_refused_with_the_nearest() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["ls", "status == \"wpi\""]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown status: 'wpi' in filter 'status == \"wpi\"'\n\
         hint: did you mean wip?\n\
         hint: use one of: proposed, accepted, wip, done, cancelled, failed\n"
    );
}

#[test]
fn a_view_naming_an_undeclared_status_fails_at_load() {
    let dir = support::scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n[views.broken]\nfilter = 'status == \"wpi\"'\n",
    );

    let (code, stderr) = run(dir.path(), &["ls"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("views.broken"), "{stderr}");
    assert!(stderr.contains("'wpi'"), "{stderr}");
    assert!(stderr.contains("hint: did you mean wip?\n"), "{stderr}");
    assert!(stderr.contains("hint: use one of: "), "{stderr}");
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
fn a_values_file_that_cannot_be_read_is_one_error_without_caused_by() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["add", "-f", "/nonexistent", "Gamma"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(!stderr.contains("Caused by"), "{stderr}");
    assert_eq!(
        stderr,
        "error: can't read values: '/nonexistent': no such file or directory\n"
    );
}

#[test]
fn a_missing_slot_names_the_flags_that_fill_it() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["add", "--name", "gamma"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: missing a required slot: 'title'\n\
         hint: pass each with --set name=value, or with -f\n\
         hint: run mt add --describe to see the slots\n"
    );
}

#[test]
fn an_unknown_template_is_a_usage_error_with_a_hint() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["add", "-t", "nope", "Gamma"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown template: 'nope'\nhint: run mt add --list to see the templates\n"
    );
}

#[test]
fn a_link_to_a_missing_task_is_one_error_and_no_summary() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["link", "alpha", "depends-on", "nope"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        stderr,
        "error: tasks/alpha.typ: edge target names no task: 'nope' of kind 'depends-on'\n\
         hint: run mt ls to see your tasks\n"
    );
}

#[test]
fn an_unknown_field_for_set_is_a_usage_error() {
    let dir = two_tasks();

    let (code, stderr) = run(dir.path(), &["set", "alpha", "titel=x"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: unknown field: 'titel'\nhint: did you mean title?\n\
         hint: run mt set --help to see the fields you can set\n"
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
