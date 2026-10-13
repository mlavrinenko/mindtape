//! `mt check` reports every problem on stderr in the error format, ends with
//! one count line, and leaves stdout to the `ok:` line.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

use support::{commit_repo, mt, scaffold_project};

fn broken_project() -> tempfile::TempDir {
    let dir = scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.rules]]\nname = \"imperative-title\"\n\
         assert = 'in_dict(\"mindtape-dev-verbs\", word(title, 1))'\n\
         message = \"start the title with an imperative verb\"\nseverity = \"error\"\n",
    );
    write_file(
        dir.path(),
        "tasks/subject.typ",
        mindtape::typst_task!(
            "(\n  title: \"Loader refactoring\",\n  status: wip(2026, 6, 1),\n)\n"
        ),
    );
    dir
}

fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = mt(dir).args(args).output().unwrap();
    (
        out.status.code(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn a_breach_and_the_count_go_to_stderr_and_stdout_stays_empty() {
    let dir = broken_project();

    let (code, stdout, stderr) = run(dir.path(), &["check"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(stdout, "", "stdout carries only the ok line");
    assert!(
        stderr.contains(
            "error: tasks/subject.typ:3: rules.imperative-title: start the title with an imperative verb\n"
        ),
        "{stderr}"
    );
    assert!(stderr.ends_with("error: 1 problem in 1 task\n"), "{stderr}");
    assert!(!stderr.contains("Error:"), "{stderr}");
}

#[test]
fn a_range_git_cannot_read_is_one_error_with_its_reason() {
    let dir = commit_repo("");

    let (code, _, stderr) = run(dir.path(), &["check", "--commits", "HEAD~5..HEAD"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.starts_with("error: can't read commit range: 'HEAD~5..HEAD': "),
        "{stderr}"
    );
    assert_eq!(stderr.lines().count(), 1, "one line, no chain: {stderr}");
}

#[test]
fn a_commit_message_problem_goes_to_stderr() {
    let dir = commit_repo(
        "[[commits.rules]]\nname = \"work-names-a-task\"\nassert = 'subject == \"fix: y\"'\n\
         message = \"name the task this commit works on\"\nseverity = \"error\"\n",
    );
    let file = dir.path().join("MSG");
    std::fs::write(&file, "feat: x\n").unwrap();

    let out = mt(dir.path())
        .args(["check", "--commit-msg"])
        .arg(&file)
        .output()
        .unwrap();

    let (stdout, stderr) = (
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert_eq!(stdout, "");
    assert!(
        stderr.contains("error: ") && stderr.contains("commits.rules.work-names-a-task"),
        "{stderr}"
    );
    assert!(
        stderr.ends_with("error: 1 commit message problem\n"),
        "{stderr}"
    );
}
