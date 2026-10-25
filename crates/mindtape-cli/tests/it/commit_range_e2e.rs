//! End-to-end tests for `mt check --commits <range>`
//! (`truth/tasks/check-commit-rules-over-a-commit-range.typ`): every
//! `[[commits.rules]]` rule run on each commit of a range, reading the
//! message, the files and the task versions from the commit and its first
//! parent.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

use support::{commit_repo, git};

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
    write_file(
        dir,
        "tasks/parser.typ",
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Write the parser\",\n  status: {status},\n  \
             priority: 3,\n)\n"
        ),
    );
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
    support::run_text(dir, &["check", "--commits", range])
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
    let offender = format!(
        "error: {}: commits.rules.flip-with-work: parser: start or close",
        short(dir, "HEAD")
    );
    assert!(text.contains(&offender), "names the commit: {text}");
    assert!(
        text.contains("note: commit: 'task: close the parser'"),
        "the breach names the rule and the task: {text}"
    );
    assert!(
        !text.contains("feat: start parsing"),
        "the commit carrying its work passes: {text}"
    );
    assert!(
        text.contains("1 commit broke commit rules"),
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
        text.contains(": Refs: `tasks/parser.typ` is a path"),
        "{text}"
    );
}

#[test]
fn a_range_git_cannot_read_fails() {
    let dir = commit_repo("");
    let (ok, text) = check_range(dir.path(), "no-such-rev..HEAD");
    assert!(!ok, "{text}");
    assert!(
        text.starts_with("error: can't read commit range: 'no-such-rev..HEAD': "),
        "names the range: {text}"
    );
}

#[test]
fn a_root_commit_is_read_against_the_empty_tree() {
    // The seed commit adds `parser` as wip: a start, from no task at all.
    let dir = commit_repo(FLIP_WITH_WORK);
    let (ok, text) = check_range(dir.path(), "HEAD");
    assert!(!ok, "{text}");
    assert!(
        text.contains(": commits.rules.flip-with-work: parser:")
            && text.contains("note: commit: 'seed'"),
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

/// The rule `mindtape:tasks` ships for the tip of a range.
const FINISH_AT_THE_TIP: &str = "[[commits.rules]]\n\
     name = \"named-tasks-finish-at-the-tip\"\n\
     scope = \"tip\"\n\
     assert = '!(status == \"wip\")'\n\
     message = \"run `mt done` and commit it with the task's last work\"\n\
     severity = \"error\"\n";

#[test]
fn a_range_naming_a_task_left_wip_fails_with_its_stem() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/lib.rs");
    commit(dir, "feat: start parsing\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(!ok, "a task still wip at the tip refuses the range: {text}");
    assert!(
        text.contains(&format!(
            "error: {}: commits.rules.named-tasks-finish-at-the-tip: parser: run `mt done`",
            short(dir, "HEAD")
        )),
        "names the tip, the rule, the stem and the advice: {text}"
    );
}

#[test]
fn a_range_naming_a_task_that_is_done_passes() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "done(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/lib.rs");
    commit(dir, "feat: parse\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(ok, "{text}");
}

#[test]
fn a_task_wip_in_the_middle_but_done_at_the_tip_passes() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/a.rs");
    commit(dir, "feat: start\n\nRefs: parser");
    write_parser(dir, "done(2026, 6, 2)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/b.rs");
    commit(dir, "feat: finish\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(ok, "only the tip's state counts: {text}");
}

#[test]
fn a_stem_named_in_two_commits_is_reported_once() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/a.rs");
    commit(dir, "feat: start\n\nRefs: parser");
    stage_file(dir, "src/b.rs");
    commit(dir, "feat: go on\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(!ok, "{text}");
    assert_eq!(
        text.matches("named-tasks-finish-at-the-tip: parser:")
            .count(),
        1,
        "{text}"
    );
}

#[test]
fn a_task_the_tip_deleted_is_not_left_wip() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/a.rs");
    commit(dir, "feat: start\n\nRefs: parser");
    git(dir, &["rm", "-q", "tasks/parser.typ"]);
    commit(dir, "task: drop the parser");

    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(ok, "a task the tip lacks cannot be wip there: {text}");
}

#[test]
fn the_pack_lets_a_fixup_through_the_hook_and_refuses_it_in_a_range() {
    let dir = commit_repo("");
    let dir = dir.path();
    let config = dir.join(".mindtape/config.toml");
    let base = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("extends = [\"mindtape:tasks\"]\n{base}")).unwrap();
    git(dir, &["add", "-A"]);
    commit(dir, "chore: extend the pack\n\nChangelog: none");
    stage_file(dir, "src/lib.rs");
    let message = "fixup! feat: list dictionaries\n";

    let (ok, text) = support::check_commit_msg(dir, message);
    assert!(ok, "the hook lets a fixup through: {text}");

    commit(dir, message);
    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(!ok, "the range must refuse the fixup: {text}");
    assert!(text.contains("no-unsquashed-fixup"), "{text}");
    assert!(text.contains("git rebase -i --autosquash"), "{text}");
}
