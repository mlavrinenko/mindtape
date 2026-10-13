//! A commit may name a task only once the committed tree holds it started or
//! finished (`truth/tasks/refuse-a-commit-whose-refs-task-is-not-started-and-an-integrate-that-leaves-one-wip.typ`).
//! The rule is this repository's own `[[commits.rules]]` entry, written with
//! the task-scope language and nothing added to it.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

use support::{check_commit_msg, commit_repo, git};

/// The same entry `.mindtape/config.toml` declares.
const REFS_TASK_STARTED: &str = "[[commits.rules]]\n\
     name = \"refs-task-is-started\"\n\
     scope = \"task\"\n\
     when = 'named && staged(exists)'\n\
     assert = 'staged(status == \"wip\" || terminal)'\n\
     message = \"flip the task to wip in the first commit that names it\"\n\
     severity = \"error\"\n";

/// Stage `tasks/parser.typ` with `status`, and one work file beside it when
/// `with_work` is set.
fn stage(dir: &Path, status: &str, with_work: bool) {
    write_file(
        dir,
        "tasks/parser.typ",
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Write the parser\",\n  status: {status},\n  \
             priority: 3,\n)\n"
        ),
    );
    git(dir, &["add", "tasks/parser.typ"]);
    if with_work {
        std::fs::create_dir_all(dir.join("src")).unwrap();
        write_file(dir, "src/lib.rs", "work\n");
        git(dir, &["add", "src/lib.rs"]);
    }
}

#[test]
fn work_naming_a_task_still_proposed_is_refused() {
    let dir = commit_repo(REFS_TASK_STARTED);
    stage(dir.path(), "proposed(2026, 6, 1)", true);
    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(!ok, "a proposed task cannot be worked on: {text}");
    assert!(
        text.contains("commits.rules.refs-task-is-started: parser: flip the task to wip"),
        "{text}"
    );
}

#[test]
fn work_naming_a_task_whose_flip_stayed_unstaged_is_refused() {
    let dir = commit_repo(REFS_TASK_STARTED);
    stage(dir.path(), "proposed(2026, 6, 1)", true);
    // The flip exists in the working tree only.
    write_file(
        dir.path(),
        "tasks/parser.typ",
        mindtape::typst_task!(
            "\n\
         #show: task.with(\n  title: \"Write the parser\",\n  status: wip(2026, 6, 2),\n  \
         priority: 3,\n)\n"
        ),
    );
    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(!ok, "an unstaged flip is not the commit's: {text}");
}

#[test]
fn work_naming_a_wip_task_passes() {
    let dir = commit_repo(REFS_TASK_STARTED);
    stage(dir.path(), "wip(2026, 6, 1)", true);
    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(ok, "{text}");
}

#[test]
fn a_decision_closing_in_a_task_only_commit_passes() {
    let dir = commit_repo(REFS_TASK_STARTED);
    stage(dir.path(), "done(2026, 6, 2)", false);
    let (ok, text) = check_commit_msg(dir.path(), "task: decide the parser\n\nRefs: parser\n");
    assert!(ok, "a staged close satisfies the rule: {text}");
}

#[test]
fn a_commit_naming_no_task_is_outside_the_rule() {
    let dir = commit_repo(REFS_TASK_STARTED);
    stage(dir.path(), "proposed(2026, 6, 1)", true);
    let (ok, text) = check_commit_msg(dir.path(), "task: file the parser\n");
    assert!(ok, "filing names no task: {text}");
}
