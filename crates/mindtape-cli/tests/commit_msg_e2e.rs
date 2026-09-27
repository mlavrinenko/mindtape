//! End-to-end tests for `mt check --commit-msg <file>`
//! (`truth/tasks/check-commit-messages-against-commit-rules.typ`): the
//! `[[commits.rules]]` a `commit-msg` hook runs over the message being
//! written and the index it is about to commit.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

use support::{git, git_commit_all, mt, project_with_tasks};

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

/// A committed-clean repository holding one task, `parser`, and `rules`
/// appended to its config.
fn repo(rules: &str) -> tempfile::TempDir {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser")]);
    let config = dir.path().join(".mindtape/config.toml");
    let base = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("{base}\n{rules}")).unwrap();
    git_commit_all(dir.path());
    dir
}

/// Stage one new file at `rel`.
fn stage(dir: &Path, rel: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "work\n").unwrap();
    git(dir, &["add", rel]);
}

/// `mt check --commit-msg` over `message`, as a hook would run it: its
/// combined output, and whether it exited zero.
fn check_msg(dir: &Path, message: &str) -> (bool, String) {
    let file = dir.join("COMMIT_MSG");
    std::fs::write(&file, message).unwrap();
    let out = mt(dir)
        .args(["check", "--commit-msg"])
        .arg(&file)
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
fn a_work_commit_naming_its_task_by_stem_passes() {
    let dir = repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "src/lib.rs");
    let (ok, text) = check_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(ok, "a stem naming a live task must pass: {text}");
}

#[test]
fn a_work_commit_with_no_trailer_breaks_the_rule() {
    let dir = repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "src/lib.rs");
    let (ok, text) = check_msg(dir.path(), "feat: parse\n\nNo trailer here.\n");
    assert!(!ok, "a missing trailer must fail an error rule: {text}");
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
    let dir = repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "src/lib.rs");
    let (ok, text) = check_msg(dir.path(), "feat: parse\n\nRefs: tasks/parser.typ\n");
    assert!(!ok, "a path in the trailer must be refused: {text}");
    assert!(
        text.contains("Refs: parser"),
        "the refusal names the stem to write: {text}"
    );
}

#[test]
fn a_trailer_naming_no_task_is_refused() {
    let dir = repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "src/lib.rs");
    let (ok, text) = check_msg(dir.path(), "feat: parse\n\nRefs: parsr\n");
    assert!(!ok, "an unknown stem must be refused: {text}");
    assert!(
        text.contains("parsr") && text.contains("names no task"),
        "the refusal names the value: {text}"
    );
}

#[test]
fn a_bad_trailer_is_refused_even_with_no_rule_declared() {
    let dir = repo("");
    stage(dir.path(), "src/lib.rs");
    let (ok, text) = check_msg(dir.path(), "feat: parse\n\nRefs: parsr\n");
    assert!(
        !ok,
        "trailer resolution is not a rule a project opts into: {text}"
    );
}

#[test]
fn a_when_that_does_not_match_passes() {
    let dir = repo(WORK_NAMES_A_TASK);
    stage(dir.path(), "tasks/lexer.typ");
    let (ok, text) = check_msg(dir.path(), "task: file the lexer\n");
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
    let dir = repo(rules);
    stage(dir.path(), "docs/guide.md");
    let (ok, text) = check_msg(dir.path(), "feat: guide\n");
    assert!(!ok, "a docs change under a feat subject breaks it: {text}");
    let (ok, text) = check_msg(dir.path(), "docs: guide\n");
    assert!(ok, "a docs subject satisfies it: {text}");
}

#[test]
fn a_task_field_in_a_commit_rule_is_refused_at_load() {
    let rules = "[[commits.rules]]\n\
         name = \"bad\"\n\
         assert = 'status == \"wip\"'\n";
    let dir = repo(rules);
    let (ok, text) = check_msg(dir.path(), "feat: x\n");
    assert!(!ok, "a task field cannot be read by a commit rule: {text}");
    assert!(text.contains("commits.rules[0].assert"), "got: {text}");
}

#[test]
fn outside_a_git_repository_the_check_fails_naming_why() {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser")]);
    let (ok, text) = check_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(!ok, "no index means nothing to check: {text}");
    assert!(text.contains("not a git repository"), "got: {text}");
}
