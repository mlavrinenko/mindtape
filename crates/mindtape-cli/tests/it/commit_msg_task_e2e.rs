//! End-to-end tests for `scope = "task"` commit rules
//! (`truth/tasks/evaluate-task-scope-commit-rules.typ`): a rule run per task
//! the commit changes or names, reading that task as of `HEAD`, the index and
//! the working tree through `head(...)`, `staged(...)` and `worktree(...)`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

use support::{check_commit_msg, commit_repo, git};

/// The task's worked example: a start or a close rides with the work it
/// claims, unless the commit is a decision's own.
const FLIP_WITH_WORK: &str = "[[commits.rules]]\n\
     name = \"flip-with-work\"\n\
     scope = \"task\"\n\
     when = 'into(\"wip\") || into(\"done\")'\n\
     assert = 'named && (!tasks_only || staged(has_tag(\"decision\")))'\n\
     message = \"start or close a task in a commit that carries its work\"\n\
     severity = \"error\"\n";

/// Rewrite `tasks/parser.typ` with `status` and any `extra` fields.
fn write_parser(dir: &Path, status: &str, extra: &str) {
    std::fs::write(
        dir.join("tasks/parser.typ"),
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Write the parser\",\n  status: {status},\n  \
             priority: 3,\n{extra})\n"
        ),
    )
    .unwrap();
}

/// Close `parser` and stage the close.
fn stage_close(dir: &Path, extra: &str) {
    write_parser(dir, "done(2026, 6, 2)", extra);
    git(dir, &["add", "tasks/parser.typ"]);
}

/// Stage one new work file.
fn stage_work(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "work\n").unwrap();
    git(dir, &["add", "src/lib.rs"]);
}

#[test]
fn a_close_only_commit_is_refused_naming_the_task() {
    let dir = commit_repo(FLIP_WITH_WORK);
    stage_close(dir.path(), "");
    let (ok, text) = check_commit_msg(dir.path(), "task: close the parser\n");
    assert!(!ok, "a close with no work must be refused: {text}");
    assert!(
        text.contains("commits.rules.flip-with-work: parser: start or close"),
        "the breach names the rule, the task and the message: {text}"
    );
}

#[test]
fn a_close_riding_with_its_work_passes() {
    let dir = commit_repo(FLIP_WITH_WORK);
    stage_close(dir.path(), "");
    stage_work(dir.path());
    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(ok, "a close beside the work it claims passes: {text}");
}

#[test]
fn a_work_commit_leaving_the_status_alone_is_outside_the_rule() {
    let dir = commit_repo(FLIP_WITH_WORK);
    write_parser(dir.path(), "wip(2026, 6, 1)", "  tags: (\"x\",),\n");
    git(dir.path(), &["add", "tasks/parser.typ"]);
    let (ok, text) = check_commit_msg(dir.path(), "task: tag the parser\n");
    assert!(ok, "no flip, so `into` never holds: {text}");
}

#[test]
fn an_estimate_added_in_the_closing_commit_is_read_through_staged() {
    let rules = "[[commits.rules]]\n\
         name = \"closed-with-estimate\"\n\
         scope = \"task\"\n\
         when = 'into(\"done\")'\n\
         assert = 'staged(has(estimate)) && !head(has(estimate))'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    stage_close(dir.path(), "  estimate: (days: 1),\n");
    let (ok, text) = check_commit_msg(dir.path(), "task: close the parser\n");
    assert!(ok, "the staged close carries the estimate: {text}");

    // The same estimate only in the working tree: the index lacks it.
    stage_close(dir.path(), "");
    write_parser(dir.path(), "done(2026, 6, 2)", "  estimate: (days: 1),\n");
    let (ok, text) = check_commit_msg(dir.path(), "task: close the parser\n");
    assert!(!ok, "an unstaged estimate is not the commit's: {text}");
    assert!(
        text.contains("commits.rules.closed-with-estimate"),
        "{text}"
    );
}

#[test]
fn a_flip_left_unstaged_warns() {
    let rules = "[[commits.rules]]\n\
         name = \"flip-unstaged\"\n\
         scope = \"task\"\n\
         when = 'worktree(status) != staged(status)'\n\
         assert = 'miss(refs)'\n\
         message = \"stage the flip with its work\"\n";
    let dir = commit_repo(rules);
    stage_work(dir.path());
    write_parser(dir.path(), "done(2026, 6, 2)", "");
    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(ok, "a warn rule reports without failing: {text}");
    assert!(
        text.contains("warning:") && text.contains("flip-unstaged: parser: stage the flip"),
        "the warning names the task: {text}"
    );
}

#[test]
fn a_deleted_task_does_not_exist_in_the_staged_state() {
    let rules = "[[commits.rules]]\n\
         name = \"no-deleting-open-tasks\"\n\
         scope = \"task\"\n\
         when = 'head(exists) && !staged(exists)'\n\
         assert = 'head(terminal)'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    git(dir.path(), &["rm", "-q", "tasks/parser.typ"]);
    let (ok, text) = check_commit_msg(dir.path(), "task: drop the parser\n");
    assert!(!ok, "an open task deleted breaks it: {text}");
    assert!(text.contains("no-deleting-open-tasks: parser"), "{text}");
}

#[test]
fn a_bare_task_field_in_a_task_rule_is_refused_at_load() {
    let rules = "[[commits.rules]]\n\
         name = \"bare\"\n\
         scope = \"task\"\n\
         assert = 'status == \"done\"'\n";
    let dir = commit_repo(rules);
    let (ok, text) = check_commit_msg(dir.path(), "feat: x\n");
    assert!(!ok, "a bare field names no state: {text}");
    assert!(
        text.contains("commits.rules[0].assert") && text.contains("staged("),
        "the refusal names the state wrappers: {text}"
    );
}

#[test]
fn a_state_wrapper_in_a_commit_rule_is_refused_at_load() {
    let rules = "[[commits.rules]]\n\
         name = \"wrapped\"\n\
         assert = 'staged(status == \"done\")'\n";
    let dir = commit_repo(rules);
    let (ok, text) = check_commit_msg(dir.path(), "feat: x\n");
    assert!(!ok, "a commit rule has no task: {text}");
    assert!(text.contains("scope = \"task\""), "{text}");
}

#[test]
fn into_naming_no_status_is_refused_at_load() {
    let rules = "[[commits.rules]]\n\
         scope = \"task\"\n\
         when = 'into(\"dnoe\")'\n\
         assert = 'named'\n";
    let dir = commit_repo(rules);
    let (ok, text) = check_commit_msg(dir.path(), "feat: x\n");
    assert!(!ok, "a typo'd status must fail load: {text}");
    assert!(text.contains("dnoe"), "{text}");
}
