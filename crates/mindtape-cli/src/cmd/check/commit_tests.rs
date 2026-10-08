//! Tests for `mt check --commit-msg`: the failure count that sums trailer
//! problems with rule breaches, the quiet flag that suppresses the success
//! line, and `warn_state_errors` printing each state that failed to evaluate.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::{run, warn_state_errors};
use crate::test_util::make_project_with_lib;

use mindtape::test_support::PRELUDE;

const TASK: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Write the parser\",\n  status: wip(2026, 6, 1),\n)\n";

/// A trailer that names no task is a ref problem, and an error-severity rule
/// the message breaks is a breach — this config makes one of each fire.
const RULE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n\
     [commits]\ntrailer = \"Refs\"\n\n\
     [[commits.rules]]\n\
     name = \"subject-prefix\"\n\
     scope = \"commit\"\n\
     when = '!tasks_only'\n\
     assert = 'matches(subject, \"^docs\")'\n\
     message = \"docs changes need a docs subject\"\n\
     severity = \"error\"\n";

/// A config with no commit rules, so a well-formed message passes.
const PASS_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[commits]\ntrailer = \"Refs\"\n";

fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

/// The committed-clean repository `--commit-msg` needs: `staged_files` reads
/// the index, so `run` only works inside a git repo. `--no-gpg-sign` keeps a
/// developer's signing setup out of the seed commit.
fn commit_all(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "t"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "--no-gpg-sign", "-q", "-m", "seed"]);
}

/// Stage a new non-task file, so `!tasks_only` holds.
fn stage(dir: &Path, rel: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b"work\n").unwrap();
    git(dir, &["add", rel]);
}

/// The failure count is the sum of unresolved-trailer problems and breached
/// rules, so a message with one of each fails with a count of two. Replacing
/// `+` with `-` would cancel them to zero and report success.
#[test]
fn the_failure_count_sums_trailer_problems_and_rule_breaches() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project_with_lib(dir.path(), PRELUDE, &[("parser.typ", TASK)], RULE_CONFIG);
    commit_all(dir.path());
    stage(dir.path(), "src/lib.rs");
    let message = dir.path().join("COMMIT_MSG");
    std::fs::write(&message, "feat: x\n\nRefs: parsr\n").unwrap();

    let result = run(&project, &message, true);
    assert!(
        result.is_err(),
        "one bad trailer plus one breached rule must fail: {result:?}"
    );
}

const QUIET_CHILD: &str = "_MT_CHECK_COMMIT_QUIET_CHILD";

/// `quiet` suppresses the success line: a valid message checked quietly prints
/// nothing. Dropping the `!` would print it exactly when quiet was asked for.
#[test]
fn quiet_suppresses_the_ok_line() {
    if std::env::var(QUIET_CHILD).is_err() {
        let out = crate::test_util::run_self_test("quiet_suppresses_the_ok_line", QUIET_CHILD);
        crate::test_util::assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            !stdout.contains("ok: commit message valid"),
            "quiet must print no success line: {stdout}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = make_project_with_lib(dir.path(), PRELUDE, &[("parser.typ", TASK)], PASS_CONFIG);
    commit_all(dir.path());
    let message = dir.path().join("COMMIT_MSG");
    std::fs::write(&message, "feat: x\n\nRefs: parser\n").unwrap();
    run(&project, &message, true).unwrap();
}

const WARN_CHILD: &str = "_MT_CHECK_COMMIT_WARN_CHILD";

/// Every state that holds a task file it cannot evaluate is reported as a
/// warning on stderr. Replacing the body with `()` would print nothing.
#[test]
fn warn_state_errors_reports_each_state() {
    if std::env::var(WARN_CHILD).is_err() {
        let out =
            crate::test_util::run_self_test("warn_state_errors_reports_each_state", WARN_CHILD);
        crate::test_util::assert_child_ok(&out);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("warning: tasks/head.typ as HEAD: boom"),
            "the HEAD state's error must be reported: {stderr}"
        );
        assert!(
            stderr.contains("warning: tasks/staged.typ as staged: broken"),
            "the staged state's error must be reported: {stderr}"
        );
        return;
    }
    let mut states = mindtape::CommitStates::default();
    states.head.errors.push(mindtape::LoadError {
        file: "tasks/head.typ".into(),
        message: "boom".to_owned(),
        ..mindtape::LoadError::default()
    });
    states.staged.errors.push(mindtape::LoadError {
        file: "tasks/staged.typ".into(),
        message: "broken".to_owned(),
        ..mindtape::LoadError::default()
    });
    warn_state_errors(&states, None);
}
