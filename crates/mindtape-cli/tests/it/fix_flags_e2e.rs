//! End-to-end tests for what `mt fix` prints: the default rows, `-v` and `-q`.
//!
//! A run prints one row per import it finds and a closing count. `--check`
//! prints the rewrites and the count, `-v` adds a row per kept import, and `-q`
//! doesn't print while the run succeeds. The exit code is the same either way.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::fix_e2e::{has, lacks, project_with, put, read, task, unchanged};
use crate::support;

/// Run `mt fix …` and return its exit code, stdout and stderr.
fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    support::parts_of(&support::mt(dir).args(args).output().unwrap())
}

/// Run `mt fix …`, assert it exited `code`, and assert it didn't print.
fn silent(dir: &Path, args: &[&str], code: i32) {
    let (got, stdout, stderr) = run(dir, args);
    assert_eq!(got, Some(code), "{stderr}");
    assert_eq!(stdout, "", "{args:?} printed: {stdout}");
    assert_eq!(stderr, "", "{args:?} printed: {stderr}");
}

#[test]
fn check_prints_the_rewrites_and_the_summary() {
    let (dir, before) = project_with("0.1.0");
    put(dir.path(), "tasks/kept.typ", "0.2.0", "Kept");

    let (code, stdout, stderr) = run(dir.path(), &["fix", "--check"]);
    assert_eq!(code, Some(1), "{stderr}");
    has(
        &stdout,
        &[
            "rewrite tasks/old.typ  0.1.0 -> 0.4.0",
            "would rewrite 1 import; 1 kept",
        ],
    );
    lacks(&stdout, &["kept    tasks/kept.typ"]);
    has(
        &stderr,
        &["error: tasks/old.typ: imports @local/mindtape:0.1.0"],
    );
    unchanged(dir.path(), "tasks/old.typ", &before);
}

#[test]
fn verbose_check_adds_a_row_per_kept_import() {
    let (dir, _) = project_with("0.1.0");
    put(dir.path(), "tasks/kept.typ", "0.2.0", "Kept");

    for flag in ["-v", "--verbose"] {
        let (code, stdout, stderr) = run(dir.path(), &["fix", "--check", flag]);
        assert_eq!(code, Some(1), "{stderr}");
        has(
            &stdout,
            &[
                "rewrite tasks/old.typ",
                "kept    tasks/kept.typ  0.2.0",
                "would rewrite 1 import; 1 kept",
            ],
        );
    }
}

#[test]
fn a_clean_check_prints_the_summary_and_a_quiet_one_prints_nothing() {
    let (dir, before) = project_with("0.2.0");

    let (code, stdout, stderr) = run(dir.path(), &["fix", "--check"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(stdout, "nothing to rewrite; 1 kept\n", "{stdout}");
    silent(dir.path(), &["fix", "--check", "-q"], 0);
    unchanged(dir.path(), "tasks/old.typ", &before);
}

#[test]
fn a_quiet_check_still_fails_an_unsupported_pin() {
    let (dir, before) = project_with("0.1.0");

    let (code, stdout, stderr) = run(dir.path(), &["fix", "--check", "-q"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(stdout, "", "a quiet check printed a row: {stdout}");
    assert!(
        stderr.contains("error: tasks/old.typ: imports @local/mindtape:0.1.0"),
        "a quiet check lost its diagnostic: {stderr}"
    );
    unchanged(dir.path(), "tasks/old.typ", &before);
}

#[test]
fn a_quiet_write_prints_nothing_and_keeps_its_edit() {
    let (dir, _) = project_with("0.1.0");

    silent(dir.path(), &["fix", "-q"], 0);
    assert_eq!(read(dir.path(), "tasks/old.typ"), task("0.4.0", "Old task"));
}
