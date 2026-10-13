//! End-to-end tests for `mt check --commit-msg <file>`
//! (`truth/tasks/check-commit-messages-against-commit-rules.typ`): the
//! `[[commits.rules]]` a `commit-msg` hook runs over the message being
//! written and the index it is about to commit.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

use support::{check_commit_msg, commit_repo, git, project_with_tasks};

/// The rule the task ships as its worked example: every commit carrying work
/// names the task it works on.
const WORK_NAMES_A_TASK: &str = "[commits]\ntrailer = \"Refs\"\n\n\
     [[commits.rules]]\n\
     name = \"work-names-a-task\"\n\
     scope = \"commit\"\n\
     when = '!tasks_only'\n\
     assert = 'has(refs)'\n\
     message = \"name the task this commit works on\"\n\
     severity = \"error\"\n";

/// Stage one new file at `rel`.
fn stage(dir: &Path, rel: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "work\n").unwrap();
    git(dir, &["add", rel]);
}

#[test]
fn a_work_commit_naming_its_task_by_stem_passes() {
    let dir = commit_repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "src/lib.rs");
    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(ok, "a stem naming a live task must pass: {text}");
}

#[test]
fn a_work_commit_with_no_trailer_breaks_the_rule() {
    let dir = commit_repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "src/lib.rs");
    let text = support::commit_msg_refusal(dir.path(), "feat: parse\n\nNo trailer here.\n");
    assert!(
        text.contains("commits.rules.work-names-a-task"),
        "the breach names its rule: {text}"
    );
    assert!(
        text.contains("name the task this commit works on"),
        "the breach carries the rule's message: {text}"
    );
}

#[test]
fn a_path_shaped_trailer_is_refused_naming_the_stem() {
    let dir = commit_repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "src/lib.rs");
    let text = support::commit_msg_refusal(dir.path(), "feat: parse\n\nRefs: tasks/parser.typ\n");
    assert!(
        text.contains("Refs: parser"),
        "the refusal names the stem to write: {text}"
    );
}

#[test]
fn a_trailer_naming_no_task_is_refused() {
    let dir = commit_repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "src/lib.rs");
    let text = support::commit_msg_refusal(dir.path(), "feat: parse\n\nRefs: parsr\n");
    assert!(
        text.contains("parsr") && text.contains("names no task"),
        "the refusal names the value: {text}"
    );
}

#[test]
fn a_bad_trailer_is_refused_even_with_no_rule_declared() {
    let dir = commit_repo("");
    stage(dir.path(), "src/lib.rs");
    support::commit_msg_refusal(dir.path(), "feat: parse\n\nRefs: parsr\n");
}

#[test]
fn a_when_that_does_not_match_passes() {
    let dir = commit_repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "tasks/lexer.typ");
    let (ok, text) = check_commit_msg(dir.path(), "task: file the lexer\n");
    assert!(
        ok,
        "a tasks-only commit is outside the rule's `when`: {text}"
    );
}

#[test]
fn a_rule_reads_the_subject_and_the_staged_files() {
    let rules = "[[commits.rules]]\n\
         name = \"docs-subject\"\n\
         when = 'matches(files, \"^docs/\")'\n\
         assert = 'matches(subject, \"^docs\")'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    stage(dir.path(), "docs/guide.md");
    support::commit_msg_refusal(dir.path(), "feat: guide\n");
    let (ok, text) = check_commit_msg(dir.path(), "docs: guide\n");
    assert!(ok, "a docs subject satisfies it: {text}");
}

#[test]
fn a_rule_counts_the_staged_files_a_glob_list_admits() {
    let rules = "[[commits.rules]]\n\
         name = \"one-source-file\"\n\
         assert = 'count(files, \"!docs/**\") <= 1'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    stage(dir.path(), "src/a.rs");
    stage(dir.path(), "docs/guide.md");
    let (ok, text) = check_commit_msg(dir.path(), "feat: a\n");
    assert!(ok, "one source file and a doc pass: {text}");
    stage(dir.path(), "src/b.rs");
    support::commit_msg_refusal(dir.path(), "feat: a\n");
}

#[test]
fn a_rename_counts_as_one_file_in_count() {
    let rules = "[dicts.source]\n\
         case = \"exact\"\n\
         values = [\"src/**\"]\n\n\
         [[commits.rules]]\n\
         name = \"one-source-file\"\n\
         assert = 'count(files, \"@source\") <= 1'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    stage(dir.path(), "src/old.rs");
    git(
        dir.path(),
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "seed"],
    );
    git(dir.path(), &["mv", "src/old.rs", "src/new.rs"]);
    let (ok, text) = check_commit_msg(dir.path(), "refactor: rename\n");
    assert!(ok, "a pure rename is one file: {text}");
    stage(dir.path(), "src/extra.rs");
    support::commit_msg_refusal(dir.path(), "refactor: rename\n");
}

#[test]
fn a_task_field_in_a_commit_rule_is_refused_at_load() {
    let rules = "[[commits.rules]]\n\
         name = \"bad\"\n\
         assert = 'status == \"wip\"'\n";
    let dir = commit_repo(rules);
    let text = support::commit_msg_refusal(dir.path(), "feat: x\n");
    assert!(text.contains("commits.rules[0].assert"), "got: {text}");
}

#[test]
fn outside_a_git_repository_the_check_fails_naming_why() {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser")]);
    let text = support::commit_msg_refusal(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(text.contains("not a git repository"), "got: {text}");
}
