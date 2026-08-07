//! Tests for the core `mt check` contract: project validity, `--quiet`,
//! header rules, and the `[tasks.comments]` review-comment policy.
//! `[[tasks.limits]]` work-in-progress caps live in `check_tests_limits.rs`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::test_support::TASK_SRC as TASK;
use mindtape::{Config, LoadOutcome, parse_config};

use crate::util;

pub(super) fn make_project(dir: &Path, files: &[(&str, &str)]) -> util::Project {
    crate::test_util::make_project(dir, files, Config::default())
}

pub(super) fn make_project_with_config(
    dir: &Path,
    files: &[(&str, &str)],
    config: Config,
) -> util::Project {
    crate::test_util::make_project(dir, files, config)
}

#[test]
fn valid_project_returns_ok() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn empty_project_returns_ok() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[]);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn load_error_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(
        dir.path(),
        &[("broken.typ", "#import \"@local/nope:9.9.9\": *\n")],
    );
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

#[test]
fn validation_violation_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n[tasks.priority]\nrequired = true",
    )
    .unwrap();
    // Task has no priority field; config requires it.
    let body = "#metadata((
      marker: \"mindtape.task\",
      fields: (title: \"Y\", status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none)),
      extras: (:),
    ))
    ";
    let project = make_project_with_config(dir.path(), &[("y.typ", body)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

const QUIET_CLEAN_CHILD: &str = "_MT_CHECK_QUIET_CLEAN_CHILD";

/// `--quiet` suppresses the `ok: N task(s) valid` summary on a clean run, but
/// the command still exits 0. Asserted from a child process, since the
/// parent's test harness owns stdout.
#[test]
fn quiet_suppresses_ok_summary_on_clean_run() {
    if std::env::var(QUIET_CLEAN_CHILD).is_err() {
        let out = crate::test_util::run_self_test(
            "quiet_suppresses_ok_summary_on_clean_run",
            QUIET_CLEAN_CHILD,
        );
        crate::test_util::assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            !stdout.contains("ok:"),
            "quiet clean run should suppress the ok summary, got: {stdout}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    assert!(super::run_with(&project, &super::CheckArgs { quiet: true }).is_ok());
}

const QUIET_VIOLATION_CHILD: &str = "_MT_CHECK_QUIET_VIOLATION_CHILD";

/// `--quiet` still reports violations and fails the command on a bad run —
/// only the success summary is suppressed.
#[test]
fn quiet_still_reports_violations_on_failing_run() {
    if std::env::var(QUIET_VIOLATION_CHILD).is_err() {
        let out = crate::test_util::run_self_test(
            "quiet_still_reports_violations_on_failing_run",
            QUIET_VIOLATION_CHILD,
        );
        crate::test_util::assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("priority"),
            "quiet should still print violation details, got: {stdout}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = make_project_with_config(
        dir.path(),
        &[("y.typ", BODY_NO_PRIORITY)],
        violation_config(),
    );
    assert!(super::run_with(&project, &super::CheckArgs { quiet: true }).is_err());
}

#[test]
fn multiple_valid_tasks_reports_count() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
    let outcome = LoadOutcome {
        tasks: project.outcome.tasks.clone(),
        errors: vec![],
    };
    // Confirm two tasks loaded successfully before asserting Ok.
    assert_eq!(outcome.tasks.len(), 2);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

const TASK_WITH_SUMMARY: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))

= Summary

Some content here.
";

#[test]
fn header_require_rule_passes_when_heading_present() {
    let dir = tempfile::tempdir().unwrap();
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK_WITH_SUMMARY)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn header_require_rule_fails_when_heading_absent() {
    let dir = tempfile::tempdir().unwrap();
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    // TASK has no headings at all
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

pub(super) const BODY_NO_PRIORITY: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"Y\", status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none)),
  extras: (:),
))
";

/// Scaffold a real project directory with a config that requires priority,
/// a `lib/` override for `@local/mindtape`, and one task file without priority.
/// The caller provides the TOML string for the config.
fn scaffold_violation_project(dir: &Path, toml: &str) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), toml).unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
    std::fs::write(dir.join("tasks/y.typ"), BODY_NO_PRIORITY).unwrap();
    // Install a local lib override so HarvestWorld can evaluate #metadata().
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(
        dir.join("lib/typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("lib/prelude.typ"),
        include_str!("../../../lib/prelude.typ"),
    )
    .unwrap();
}

pub(super) fn violation_config() -> Config {
    parse_config(
        "\
[[tasks.status]]
value = \"wip\"
[[tasks.status]]
value = \"done\"
terminal = true
[tasks.priority]
required = true
",
    )
    .unwrap()
}

#[test]
fn run_with_prints_violation_and_fails() {
    if std::env::var("_MT_CHECK_RUN").is_err() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .env("_MT_CHECK_RUN", "1")
            .args(["run_with_prints_violation_and_fails", "--nocapture"])
            .output()
            .unwrap();
        // Child exits 0 because its assertion passes (violation is detected).
        // What matters: stdout must contain violation details. A print_violation
        // replaced with () would print nothing, so the assertion fails.
        assert!(output.status.success(), "child should pass its assertion");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("priority"),
            "run_with should print violation details"
        );
        return;
    }
    // Child: recreate the violation project and call run_with.
    let dir = tempfile::tempdir().unwrap();
    let project = make_project_with_config(
        dir.path(),
        &[("y.typ", BODY_NO_PRIORITY)],
        violation_config(),
    );
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

/// Verify that `run()` loads the project from disk and validates it.
/// A `run` replaced with `Ok(())` would exit 0 even with violations.
const VIOLATION_TOML: &str = "\
[tasks]
roots = [\"tasks\"]

[[tasks.status]]
value = \"wip\"
[[tasks.status]]
value = \"done\"
terminal = true
[tasks.priority]
required = true
";

#[test]
fn run_validates_from_disk() {
    if std::env::var("_MT_CHECK_RUN_FROM_DISK").is_err() {
        let dir = tempfile::tempdir().unwrap();
        scaffold_violation_project(dir.path(), VIOLATION_TOML);

        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .current_dir(dir.path())
            .env("_MT_CHECK_RUN_FROM_DISK", "1")
            .args(["run_validates_from_disk", "--nocapture"])
            .output()
            .unwrap();
        // run() should fail on an invalid project. If replaced with Ok(()),
        // it would exit 0.
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "child should pass its assertion (violation detected)"
        );
        assert!(
            stdout.contains("priority"),
            "run() should report priority violation"
        );
        return;
    }
    // Child: call run() from cwd (which is the temp project).
    let result = super::run(&super::CheckArgs::default());
    assert!(result.is_err(), "run() should fail on missing priority");
}

#[test]
fn invalid_header_rule_regex_fails_with_config_error() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.body.headers.rules]]\npath = [{ match = \"[bad\" }]\naction = \"forbid\"\n",
    )
    .unwrap();
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

// --- `[tasks.comments]` review-comment policy ----------------------------
//
// Parse-plane precision (a `//` inside a string literal or a raw block is not
// a comment) is pinned in `typst_cst`'s own tests and again over
// `mindtape::check_comments` in `config_comments_tests.rs`. What matters at
// this layer is the command's contract: which policy fails `mt check`, and
// that a violation surfaces the file, the line, and the comment text.

/// `TASK` with a trailing line comment dropped after the `task.with(...)`-style
/// `#metadata(...)` call — the whole file, not only the body.
const TASK_WITH_COMMENT: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
// no, use in_view() instead
";

fn comments_project(dir: &Path, files: &[(&str, &str)], policy: &str) -> util::Project {
    let toml = format!("[tasks.comments]\npolicy = \"{policy}\"\n");
    make_project_with_config(dir, files, parse_config(&toml).unwrap())
}

#[test]
fn default_policy_allow_ignores_a_comment() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("a.typ", TASK_WITH_COMMENT)]);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn forbid_policy_passes_a_file_with_no_comments() {
    let dir = tempfile::tempdir().unwrap();
    let project = comments_project(dir.path(), &[("a.typ", TASK)], "forbid");
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn warn_policy_does_not_fail_the_command() {
    let dir = tempfile::tempdir().unwrap();
    let project = comments_project(dir.path(), &[("a.typ", TASK_WITH_COMMENT)], "warn");
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

const COMMENTS_CHILD: &str = "_MT_CHECK_COMMENTS_CHILD";

/// `forbid` fails the command, and the printed violation names the file, the
/// line, and the comment text — asserted from a child process, since the
/// parent's test harness owns stdout.
#[test]
fn forbid_policy_fails_and_reports_file_line_and_text() {
    if std::env::var(COMMENTS_CHILD).is_err() {
        let out = crate::test_util::run_self_test(
            "forbid_policy_fails_and_reports_file_line_and_text",
            COMMENTS_CHILD,
        );
        crate::test_util::assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        for expected in ["a.typ:10", "comments", "no, use in_view() instead"] {
            assert!(
                stdout.contains(expected),
                "missing {expected:?} in: {stdout}"
            );
        }
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = comments_project(dir.path(), &[("a.typ", TASK_WITH_COMMENT)], "forbid");
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}
