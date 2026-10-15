//! End-to-end battery for `mt set` — writing one core field of an existing
//! task from the CLI.
//!
//! Lives here (not an inline `#[path]` unit test) so every arm goes through the
//! real prelude, a real project on disk and the compiled binary: what a field
//! write is *for* is the file it leaves behind, and the two properties worth
//! pinning — the written literal re-evaluates, and a refused write leaves the
//! file byte-identical — are only observable end to end.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use crate::support;

/// A bounded priority scale (so an out-of-range value is refused by the
/// project rather than by `set` itself) and an allow-list of tags.
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n\
[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\n\n\
[[tasks.status]]\nvalue = \"wip\"\n\n\
[[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// The subject every arm writes to: a title, a status, and one tag already in
/// place, so both the insert and the replace path have something to act on.
const SUBJECT: &str = mindtape::typst_task!(
    "(\n  \
title: \"Wire OAuth\",\n  \
status: wip(2026, 6, 1),\n  \
tags: (\"backend\",),\n\
)\n\n== Summary\n\nBody.\n"
);

fn scaffold() -> tempfile::TempDir {
    support::scaffold_config(CONFIG, &[("wire-oauth.typ", SUBJECT)])
}

fn read(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("tasks/wire-oauth.typ")).unwrap()
}

/// `mt set <task> <token>`, asserting success and returning the file after it.
fn set_ok(dir: &Path, token: &str) -> String {
    support::run_ok(dir, &["set", "wire-oauth", token]);
    read(dir)
}

/// `mt set <task> <token>`, asserting refusal and returning stderr.
fn set_err(dir: &Path, token: &str) -> String {
    let assert = support::mt(dir)
        .args(["set", "wire-oauth", token])
        .assert()
        .failure();
    String::from_utf8_lossy(&assert.get_output().stderr).into_owned()
}

/// One arm per literal shape `set` renders: a quoted string, a bare number, an
/// `on(...)` date. All three land in the same file, so the run also proves
/// repeated writes compose rather than clobbering each other.
#[test]
fn writes_a_scalar_field_of_each_shape() {
    let dir = scaffold();

    let written = set_ok(dir.path(), "priority=8");
    assert!(written.contains("priority: 8"), "{written}");

    let written = set_ok(dir.path(), "due=2026-07-01");
    assert!(written.contains("due: on(2026, 7, 1)"), "{written}");

    let written = set_ok(dir.path(), "assignee=ada");
    assert!(written.contains("assignee: \"ada\""), "{written}");

    // The whole point of writing through `mt`: what lands still validates.
    support::check_ok(dir.path());
}

/// A field already present is replaced, not doubled.
#[test]
fn replaces_an_existing_value() {
    let dir = scaffold();

    let written = set_ok(dir.path(), "title=Wire OAuth properly");
    assert!(
        written.contains("title: \"Wire OAuth properly\""),
        "{written}"
    );
    assert!(!written.contains("\"Wire OAuth\","), "{written}");
}

/// `+=` appends to a list field, keeping what is already there; a second run of
/// the same append writes nothing at all.
#[test]
fn appends_to_a_list_field_and_is_idempotent() {
    let dir = scaffold();

    let written = set_ok(dir.path(), "tags+=auth");
    assert!(
        written.contains("tags: (\"backend\", \"auth\")"),
        "{written}"
    );

    let again = set_ok(dir.path(), "tags+=auth");
    assert_eq!(written, again, "a re-run must write nothing");
}

/// `=` on a list field replaces the whole list rather than appending to it.
#[test]
fn plain_assignment_replaces_a_list_field() {
    let dir = scaffold();

    let written = set_ok(dir.path(), "tags=frontend");
    assert!(written.contains("tags: (\"frontend\",)"), "{written}");
    assert!(!written.contains("backend"), "{written}");
}

/// The property the whole `pre_write` detour buys: a value the project refuses
/// leaves the file exactly as it was, so there is nothing to repair.
#[test]
fn a_refused_value_leaves_the_file_byte_identical() {
    let dir = scaffold();
    let before = read(dir.path());

    let stderr = set_err(dir.path(), "priority=99");
    assert!(stderr.contains("priority"), "{stderr}");
    assert_eq!(read(dir.path()), before, "a refused write must not land");
}

/// An unknown field is refused by name, with a suggestion and the writable
/// fields listed — the way an unknown `--set` key already is.
#[test]
fn an_unknown_field_is_refused_by_name() {
    let dir = scaffold();
    let before = read(dir.path());

    let stderr = set_err(dir.path(), "titel=nope");
    assert!(
        stderr.starts_with("error: unknown field: 'titel'\n"),
        "{stderr}"
    );
    assert!(stderr.contains("hint: did you mean title?\n"), "{stderr}");
    assert!(stderr.contains("hint: run mt set --help"), "{stderr}");
    assert_eq!(read(dir.path()), before);
}

/// A core field `set` deliberately does not write names the command that does,
/// rather than reporting it as a typo.
#[test]
fn a_field_with_its_own_command_names_that_command() {
    let dir = scaffold();

    let stderr = set_err(dir.path(), "status=done");
    assert!(stderr.contains("mt flip"), "{stderr}");

    let stderr = set_err(dir.path(), "links=other.typ");
    assert!(stderr.contains("mt link"), "{stderr}");
}

/// A value of the wrong shape for the field is refused before anything is
/// evaluated, naming the field and what it takes.
#[test]
fn a_wrong_shaped_value_is_refused_naming_the_field() {
    let dir = scaffold();

    let stderr = set_err(dir.path(), "priority=high");
    assert!(stderr.contains("priority"), "{stderr}");

    let stderr = set_err(dir.path(), "due=next tuesday");
    assert!(stderr.contains("YYYY-MM-DD"), "{stderr}");

    // `+=` is a list operator; a scalar field says so instead of appending.
    let stderr = set_err(dir.path(), "priority+=8");
    assert!(stderr.contains("list"), "{stderr}");
}

/// Markup-hazardous text survives the round trip: the value is a Typst string
/// literal, so nothing in it is evaluated on the way back in.
#[test]
fn a_hazardous_string_value_survives_the_round_trip() {
    let dir = scaffold();

    support::run_ok(
        dir.path(),
        &[
            "set",
            "wire-oauth",
            &format!("kind={}", support::MARKUP_HAZARDS),
        ],
    );

    let listed = support::mt(dir.path())
        .args(["ls", "", "--json"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&listed.get_output().stdout).into_owned();
    assert!(stdout.contains(support::MARKUP_HAZARDS), "{stdout}");
}

/// A refused write opens at the offending field's line, like `mt check`.
#[test]
fn a_refused_set_reports_the_file_and_line_of_the_breach() {
    let dir = scaffold();
    let stderr = set_err(dir.path(), "priority=99");
    assert!(stderr.contains("error: tasks/wire-oauth.typ:"), "{stderr}");
}

/// A refused write that inserts a field has no line of its own on disk, so the
/// breach opens at the `task.with(…)` call that is there, not at a line of the
/// unwritten candidate.
#[test]
fn a_refused_insert_points_at_the_call_on_disk() {
    let dir = scaffold();
    let stderr = set_err(dir.path(), "priority=99");
    assert!(
        stderr.contains("error: tasks/wire-oauth.typ:3: priority:"),
        "{stderr}"
    );
}

/// A refused write that replaces a field opens at that field's line on disk.
#[test]
fn a_refused_replace_points_at_the_field_on_disk() {
    let dir = scaffold();
    let written = set_ok(dir.path(), "priority=5");
    let line = written
        .lines()
        .position(|text| text.trim_start().starts_with("priority:"))
        .unwrap()
        + 1;
    // Put a long note ahead of it so the candidate's own line would differ.
    let padded = written.replacen("title:", "// a\n  // b\n  title:", 1);
    mindtape::write_atomic(&dir.path().join("tasks/wire-oauth.typ"), padded.as_bytes()).unwrap();
    let stderr = set_err(dir.path(), "priority=99");
    let expected = format!("error: tasks/wire-oauth.typ:{}: priority:", line + 2);
    assert!(stderr.contains(&expected), "{stderr}\n{padded}");
}

/// A task whose show rule is aliased has no literal `task.with(...)` call for
/// the field write to land in. `mt set` refuses it by name, the file's own
/// path in the message, and leaves the task byte-identical.
#[test]
fn a_task_with_an_aliased_call_is_refused_and_left_untouched() {
    let dir = scaffold();
    let body = support::write_aliased_task(dir.path(), "wire-oauth");

    let stderr = set_err(dir.path(), "title=Renamed");
    assert!(
        stderr.contains("can't find the task.with") && stderr.contains("wire-oauth.typ"),
        "{stderr}"
    );
    assert_eq!(read(dir.path()), body, "a refused write must not land");
}
