//! Tests for `[[tasks.limits]]` work-in-progress caps. `TASK` is a `wip`
//! task, and identity is the file stem, so writing it twice puts two tasks in
//! the capped set. Under/at/over-cap counting is pinned in
//! `mindtape::config_limits`; what matters here is the command's contract —
//! which severity fails `mt check`, and what it prints. Core validity,
//! headers, and comments-policy tests live in `check_tests.rs`, whose
//! `make_project_with_config` fixture this shares.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::parse_config;
use mindtape::test_support::TASK_SRC as TASK;

use super::tests::make_project_with_config;
use crate::util;

const LIMIT_WARN_TOML: &str = "\
[[tasks.limits]]
filter = 'status == \"wip\"'
max = 1
";

const LIMIT_ERROR_TOML: &str = "\
[[tasks.limits]]
filter = 'status == \"wip\"'
max = 1
severity = \"error\"
message = \"close the open one first\"
";

fn limited_project(dir: &Path, files: &[(&str, &str)], toml: &str) -> util::Project {
    make_project_with_config(dir, files, parse_config(toml).unwrap())
}

#[test]
fn limit_breach_at_warn_severity_still_passes() {
    let dir = tempfile::tempdir().unwrap();
    let project = limited_project(
        dir.path(),
        &[("a.typ", TASK), ("b.typ", TASK)],
        LIMIT_WARN_TOML,
    );
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn limit_breach_at_error_severity_fails() {
    let dir = tempfile::tempdir().unwrap();
    let project = limited_project(
        dir.path(),
        &[("a.typ", TASK), ("b.typ", TASK)],
        LIMIT_ERROR_TOML,
    );
    let err = super::run_with(&project, &super::CheckArgs::default()).unwrap_err();
    assert!(
        err.to_string().contains("1 limit breach(es)"),
        "failure should name the breach count: {err}"
    );
}

#[test]
fn a_respected_limit_passes_at_error_severity() {
    let dir = tempfile::tempdir().unwrap();
    let project = limited_project(dir.path(), &[("a.typ", TASK)], LIMIT_ERROR_TOML);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn invalid_limit_filter_fails_with_config_error() {
    let dir = tempfile::tempdir().unwrap();
    let project = limited_project(
        dir.path(),
        &[("a.typ", TASK)],
        "[[tasks.limits]]\nfilter = 'nonesuch == \"x\"'\nmax = 0\n",
    );
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

const LIMIT_CHILD: &str = "_MT_CHECK_LIMIT_CHILD";

/// The breach report itself: severity prefix, count, cap, advice, and one line
/// per offending task. Asserted from a child process, since the parent's test
/// harness owns stdout.
#[test]
fn run_with_prints_the_breach_and_its_tasks() {
    if std::env::var(LIMIT_CHILD).is_err() {
        let out = crate::test_util::run_self_test(
            "run_with_prints_the_breach_and_its_tasks",
            LIMIT_CHILD,
        );
        crate::test_util::assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        for expected in [
            "error: 2 task(s) match",
            "status == \"wip\"",
            "max 1",
            "close the open one first",
            "  - a",
            "  - b",
        ] {
            assert!(
                stdout.contains(expected),
                "missing {expected:?} in: {stdout}"
            );
        }
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = limited_project(
        dir.path(),
        &[("a.typ", TASK), ("b.typ", TASK)],
        LIMIT_ERROR_TOML,
    );
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}
