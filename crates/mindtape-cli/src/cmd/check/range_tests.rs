//! Regression tests for `cmd::check::range::run`'s per-commit reporting.
//!
//! The decisions pinned here are invisible to an exit-code-only test: a commit
//! whose either state holds a task file that will not evaluate is still
//! printed with its warnings, while one that fails only a `severity = "warn"`
//! rule is printed but does not break the run. Driving the real binary is the
//! only way to see the printed report, the same reason `cmd/rm/cli_tests.rs` spawns
//! `mt`.

#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use assert_cmd::Command;
use swhid_mint::test_support::{git, repo_with};

/// A task file that harvests to a marker and passes the floor's validation —
/// the seed every range builds on.
const VALID: &str = "#metadata((\n  marker: \"mindtape.task\",\n  fields: (\n    title: \"Ok\",\n    \
                     status: (name: \"wip\", date: none, note: none),\n  ),\n  extras: (:),\n))\n";

/// Source that fails evaluation, so the state holding it records a load error.
fn broken(tag: &str) -> String {
    format!("#let x = (\n// {tag}\n")
}

/// A repository with `files` committed, plus a `.mindtape/config.toml` holding
/// `config` (left uncommitted so it never enters a checked commit).
fn scaffold(files: &[(&str, &str)], config: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let (dir, root) = repo_with(files);
    std::fs::create_dir_all(root.join(".mindtape")).unwrap();
    write_file(&root, ".mindtape/config.toml", config);
    (dir, root)
}

/// Commit the working tree's `tasks/bad.typ` change.
fn commit_bad(root: &Path, contents: &str, message: &str) {
    write_file(root, "tasks/bad.typ", contents);
    git(root, &["add", "tasks/bad.typ"]);
    git(
        root,
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
    );
}

/// `mt check --commits <range>` in `root`: whether it exited zero, and its
/// combined output.
fn check(root: &Path, range: &str, quiet: bool) -> (bool, String) {
    let mut command = Command::cargo_bin("mt").unwrap();
    command
        .current_dir(root)
        .env(mindtape::CONFIG_HOME_ENV, root.join("no-global-config"))
        .env("HOME", root)
        .env_remove("MINDTAPE_NOW")
        .env_remove("MINDTAPE_WORKTREE")
        .env_remove("VISUAL");
    let mut args = vec!["check"];
    if quiet {
        args.push("--quiet");
    }
    args.extend(["--commits", range]);
    let out = command.args(args).output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

const NO_RULES: &str = "[tasks]\nroots = [\"tasks\"]\n";

// --- line 42: has_errors ---
//
// A state load error must report the commit (with its `warn:` lines) but not
// count it as broken. The count is `head.errors.len() + staged.errors.len()`;
// the first test has an error on one side only, so a `*` mutant reads zero and
// silently skips the commit.

#[test]
fn one_sided_state_error_still_reports_the_commit() {
    let (_dir, root) = scaffold(
        &[("tasks/bad.typ", VALID), ("seed.txt", "seed\n")],
        NO_RULES,
    );
    commit_bad(&root, &broken("staged only"), "break: staged");

    let (ok, text) = check(&root, "HEAD~1..HEAD", false);

    assert!(
        ok,
        "a state load error alone does not break the range: {text}"
    );
    assert!(
        text.contains("warning:") && text.contains("break: staged"),
        "the commit holding an unevaluable task must be reported: {text}"
    );
}

#[test]
fn errors_in_both_states_still_report_the_commit() {
    let (_dir, root) = scaffold(
        &[("tasks/bad.typ", VALID), ("seed.txt", "seed\n")],
        NO_RULES,
    );
    commit_bad(&root, &broken("parent"), "break: parent");
    commit_bad(&root, &broken("commit"), "break: commit");

    let (ok, text) = check(&root, "HEAD~1..HEAD", false);

    assert!(ok, "{text}");
    assert!(
        text.contains("warning:") && text.contains("break: commit"),
        "both states error; the commit must still be reported: {text}"
    );
}

// --- line 54: a warn-severity breach is reported, not fatal ---

#[test]
fn a_warning_breach_does_not_break_the_range() {
    let config = format!(
        "{NO_RULES}\n[[commits.rules]]\nname = \"warn-me\"\n\
         when = 'matches(subject, \"^warn\")'\nassert = 'matches(subject, \"zzz\")'\nseverity = \"warn\"\n"
    );
    let (_dir, root) = scaffold(&[("tasks/ok.typ", VALID), ("seed.txt", "seed\n")], &config);
    git(
        &root,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "warn: please look",
        ],
    );

    let (ok, text) = check(&root, "HEAD~1..HEAD", false);

    assert!(ok, "a warn-severity breach must not fail the range: {text}");
    assert!(
        text.contains("warning:") && text.contains("warn-me"),
        "the warning is still reported: {text}"
    );
    assert!(text.contains("ok:"), "{text}");
}

// --- line 61: --quiet suppresses the success summary ---

#[test]
fn a_clean_range_prints_the_success_summary() {
    let (_dir, root) = scaffold(&[("tasks/ok.typ", VALID), ("seed.txt", "seed\n")], NO_RULES);

    let (ok, text) = check(&root, "HEAD", false);
    assert!(ok, "{text}");
    assert!(text.contains("ok:"), "the summary is printed: {text}");

    let (ok, text) = check(&root, "HEAD", true);
    assert!(ok, "{text}");
    assert!(
        !text.contains("ok:"),
        "--quiet suppresses the summary: {text}"
    );
}
