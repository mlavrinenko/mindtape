//! End-to-end tests for `mt check --commits <range>`
//! (`truth/tasks/check-commit-rules-over-a-commit-range.typ`): every
//! `[[commits.rules]]` rule run on each commit of a range, reading the
//! message, the files and the task versions from the commit and its first
//! parent.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

mod support;

use support::{commit_repo, git, mt};

/// The flip-with-work rule `commit_msg_task_e2e.rs` pins for the hook.
const FLIP_WITH_WORK: &str = "[[commits.rules]]\n\
     name = \"flip-with-work\"\n\
     scope = \"task\"\n\
     when = 'into(\"wip\") || into(\"done\")'\n\
     assert = 'named && !tasks_only'\n\
     message = \"start or close a task in a commit that carries its work\"\n\
     severity = \"error\"\n";

/// Rewrite `tasks/parser.typ` with `status`.
fn write_parser(dir: &Path, status: &str) {
    std::fs::write(
        dir.join("tasks/parser.typ"),
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Write the parser\",\n  status: {status},\n  \
             priority: 3,\n)\n"
        ),
    )
    .unwrap();
}

/// Write `path` under `dir` and stage it.
fn stage_file(dir: &Path, path: &str) {
    let file = dir.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, format!("{path}\n")).unwrap();
    git(dir, &["add", path]);
}

/// Commit everything staged with `message`.
fn commit(dir: &Path, message: &str) {
    git(
        dir,
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
    );
}

/// The abbreviated id of `rev`.
fn short(dir: &Path, rev: &str) -> String {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--short", rev])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// `mt check --commits <range>`: whether it exited zero, and its output.
fn check_range(dir: &Path, range: &str) -> (bool, String) {
    let out = mt(dir)
        .args(["check", "--commits", range])
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

#[test]
fn a_range_with_one_offending_commit_fails_naming_it() {
    let dir = commit_repo(FLIP_WITH_WORK);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/lib.rs");
    commit(dir, "feat: start parsing\n\nRefs: parser");
    write_parser(dir, "done(2026, 6, 2)");
    git(dir, &["add", "tasks/parser.typ"]);
    commit(dir, "task: close the parser");

    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(!ok, "the close-only commit must fail the range: {text}");
    let offender = format!("{} task: close the parser\n", short(dir, "HEAD"));
    assert!(text.contains(&offender), "names the commit: {text}");
    assert!(
        text.contains("  error: commits.rules.flip-with-work: parser: start or close"),
        "the breach names the rule and the task: {text}"
    );
    assert!(
        !text.contains("feat: start parsing"),
        "the commit carrying its work passes: {text}"
    );
    assert!(
        text.contains("1 commit(s) broke commit rules"),
        "counts commits: {text}"
    );

    let (ok, text) = check_range(dir, "HEAD~2..HEAD~1");
    assert!(ok, "the work commit alone passes: {text}");
}

#[test]
fn a_merge_is_checked_against_its_first_parent_only() {
    let rules = "[[commits.rules]]\n\
         name = \"merge-leaves-a-alone\"\n\
         when = 'matches(subject, \"^merge\")'\n\
         assert = '!matches(files, \"^src/a\")'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    let dir = dir.path();
    git(dir, &["checkout", "-q", "-b", "side"]);
    stage_file(dir, "src/b.rs");
    commit(dir, "feat: b");
    git(dir, &["checkout", "-q", "-"]);
    stage_file(dir, "src/a.rs");
    commit(dir, "feat: a");
    git(
        dir,
        &[
            "-c",
            "commit.gpgsign=false",
            "merge",
            "-q",
            "--no-ff",
            "-m",
            "merge side",
            "side",
        ],
    );
    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(
        ok,
        "against its first parent the merge adds only src/b: {text}"
    );
}

#[test]
fn a_rule_reading_the_working_tree_is_skipped_and_reported() {
    let rules = "[[commits.rules]]\n\
         name = \"flip-unstaged\"\n\
         scope = \"task\"\n\
         when = 'worktree(status) != staged(status)'\n\
         assert = 'miss(refs)'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    let dir = dir.path();
    write_parser(dir, "done(2026, 6, 2)");
    git(dir, &["add", "tasks/parser.typ"]);
    commit(dir, "feat: close\n\nRefs: parser");
    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(ok, "a skipped rule cannot fail the range: {text}");
    assert!(
        text.contains("skip: commits.rules.flip-unstaged"),
        "the skip names the rule: {text}"
    );
}

#[test]
fn a_trailer_naming_no_task_fails_the_commit() {
    let dir = commit_repo("");
    let dir = dir.path();
    stage_file(dir, "src/lib.rs");
    commit(dir, "feat: parse\n\nRefs: tasks/parser.typ");
    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(!ok, "a path trailer is refused in a range too: {text}");
    assert!(
        text.contains("  error: Refs: `tasks/parser.typ` is a path"),
        "{text}"
    );
}

#[test]
fn a_range_git_cannot_read_fails() {
    let dir = commit_repo("");
    let (ok, text) = check_range(dir.path(), "no-such-rev..HEAD");
    assert!(!ok, "{text}");
    assert!(text.contains("--commits"), "names the flag: {text}");
}

#[test]
fn a_root_commit_is_read_against_the_empty_tree() {
    // The seed commit adds `parser` as wip: a start, from no task at all.
    let dir = commit_repo(FLIP_WITH_WORK);
    let (ok, text) = check_range(dir.path(), "HEAD");
    assert!(!ok, "{text}");
    assert!(
        text.contains("seed\n  error: commits.rules.flip-with-work: parser:"),
        "{text}"
    );
}

#[test]
fn each_state_holds_a_task_once() {
    // `parser` differs from HEAD in the parent and is changed by the commit:
    // listed twice, it would load twice.
    let rules = "[[commits.rules]]\n\
         name = \"one-parser\"\n\
         scope = \"task\"\n\
         assert = 'staged(count(priority == 3) == 1) && head(count(priority == 3) == 1)'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 2)");
    git(dir, &["add", "tasks/parser.typ"]);
    commit(dir, "task: edit\n\nRefs: parser");
    write_parser(dir, "done(2026, 6, 3)");
    git(dir, &["add", "tasks/parser.typ"]);
    commit(dir, "task: close\n\nRefs: parser");
    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(ok, "{text}");
}
