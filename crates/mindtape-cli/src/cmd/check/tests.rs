//! Tests for the core `mt check` contract: project validity, `--quiet`,
//! loading, and the `[tasks.comments]` review-comment policy. The
//! header/content body-rule cluster lives in the `cmd/check/tests_rules.rs`
//! sibling.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;
use std::path::Path;

use mindtape::test_support::TASK_SRC as TASK;
use mindtape::{Config, LoadOutcome, parse_config};

use crate::util;

pub(super) fn make_project(files: &[(&str, &str)]) -> (tempfile::TempDir, util::Project) {
    crate::test_util::make_project(files, Config::default())
}

pub(super) fn make_project_with_config(
    files: &[(&str, &str)],
    config: Config,
) -> (tempfile::TempDir, util::Project) {
    crate::test_util::make_project(files, config)
}

#[test]
fn valid_project_returns_ok() {
    let (_dir, project) = make_project(&[("x.typ", TASK)]);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn empty_project_returns_ok() {
    let (_dir, project) = make_project(&[]);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn load_error_returns_err() {
    let (_dir, project) = make_project(&[("broken.typ", "#import \"@local/nope:9.9.9\": *\n")]);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

#[test]
fn validation_violation_returns_err() {
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
    let (_dir, project) = make_project_with_config(&[("y.typ", body)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

const QUIET_CLEAN_CHILD: &str = "_MT_CHECK_QUIET_CLEAN_CHILD";

/// `--quiet` suppresses the `ok: N tasks` summary on a clean run, but
/// the command still exits 0. Asserted from a child process, since the
/// parent's test harness owns stdout.
#[test]
fn quiet_suppresses_ok_summary_on_clean_run() {
    if let Some(stdout) = mindtape::test_support::rerun_isolated_stdout(QUIET_CLEAN_CHILD) {
        assert!(
            !stdout.contains("ok:"),
            "quiet clean run should suppress the ok summary, got: {stdout}"
        );
        return;
    }
    let (_dir, project) = make_project(&[("x.typ", TASK)]);
    assert!(
        super::run_with(
            &project,
            &super::CheckArgs {
                quiet: true,
                ..Default::default()
            }
        )
        .is_ok()
    );
}

const QUIET_VIOLATION_CHILD: &str = "_MT_CHECK_QUIET_VIOLATION_CHILD";

/// `--quiet` still reports violations and fails the command on a bad run —
/// only the success summary is suppressed.
#[test]
fn quiet_still_reports_violations_on_failing_run() {
    if let Some(stderr) = mindtape::test_support::rerun_isolated_stderr(QUIET_VIOLATION_CHILD) {
        assert!(
            stderr.contains("priority"),
            "quiet should still print violation details, got: {stderr}"
        );
        return;
    }
    let (_dir, project) =
        make_project_with_config(&[("y.typ", BODY_NO_PRIORITY)], violation_config());
    assert!(
        super::run_with(
            &project,
            &super::CheckArgs {
                quiet: true,
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn multiple_valid_tasks_reports_count() {
    let (_dir, project) = make_project(&[("a.typ", TASK), ("b.typ", TASK)]);
    let outcome = LoadOutcome {
        tasks: project.outcome.tasks.clone(),
        errors: vec![],
        ..LoadOutcome::default()
    };
    // Confirm two tasks loaded successfully before asserting Ok.
    assert_eq!(outcome.tasks.len(), 2);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
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
    write_file(dir, ".mindtape/config.toml", toml);
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
    write_file(dir, "tasks/y.typ", BODY_NO_PRIORITY);
    // Install a local lib override so HarvestWorld can evaluate #metadata().
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    write_file(
        dir,
        "lib/typst.toml",
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    );
    write_file(dir, "lib/prelude.typ", mindtape::test_support::PRELUDE);
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
        let output = mindtape::test_support::run_self_test(
            &mindtape::test_support::current_test_name(),
            "_MT_CHECK_RUN",
            |_| {},
        );
        // Child exits 0 because its assertion passes (violation is detected).
        // What matters: stderr must contain violation details. A print_violation
        // replaced with () would print nothing, so the assertion fails.
        assert!(output.status.success(), "child should pass its assertion");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("priority"),
            "run_with should print violation details"
        );
        return;
    }
    // Child: recreate the violation project and call run_with.
    let (_dir, project) =
        make_project_with_config(&[("y.typ", BODY_NO_PRIORITY)], violation_config());
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
        let dir = tempdir();
        scaffold_violation_project(dir.path(), VIOLATION_TOML);

        let output = mindtape::test_support::run_self_test(
            &mindtape::test_support::current_test_name(),
            "_MT_CHECK_RUN_FROM_DISK",
            |child| {
                child.current_dir(dir.path());
            },
        );
        // run() should fail on an invalid project. If replaced with Ok(()),
        // it would exit 0.
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "child should pass its assertion (violation detected)"
        );
        assert!(
            stderr.contains("priority"),
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
    let config = parse_config(
        "[[tasks.body.headers.rules]]\npath = [{ match = \"[bad\" }]\naction = \"forbid\"\n",
    )
    .unwrap();
    let (_dir, project) = make_project_with_config(&[("a.typ", TASK)], config);
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

fn comments_project(files: &[(&str, &str)], policy: &str) -> (tempfile::TempDir, util::Project) {
    let toml = format!("[tasks.comments]\npolicy = \"{policy}\"\n");
    make_project_with_config(files, parse_config(&toml).unwrap())
}

#[test]
fn default_policy_allow_ignores_a_comment() {
    let (_dir, project) = make_project(&[("a.typ", TASK_WITH_COMMENT)]);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn forbid_policy_passes_a_file_with_no_comments() {
    let (_dir, project) = comments_project(&[("a.typ", TASK)], "forbid");
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn warn_policy_does_not_fail_the_command() {
    let (_dir, project) = comments_project(&[("a.typ", TASK_WITH_COMMENT)], "warn");
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

const COMMENTS_CHILD: &str = "_MT_CHECK_COMMENTS_CHILD";

/// `forbid` fails the command, and the printed violation names the file, the
/// line, and the comment text — asserted from a child process, since the
/// parent's test harness owns stdout.
#[test]
fn forbid_policy_fails_and_reports_file_line_and_text() {
    if let Some(stderr) = mindtape::test_support::rerun_isolated_stderr(COMMENTS_CHILD) {
        for expected in ["a.typ:10", "comments", "no, use in_view() instead"] {
            assert!(
                stderr.contains(expected),
                "missing {expected:?} in: {stderr}"
            );
        }
        return;
    }
    let (_dir, project) = comments_project(&[("a.typ", TASK_WITH_COMMENT)], "forbid");
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

fn breach(severity: mindtape::Severity, line: usize) -> mindtape::Violation {
    mindtape::Violation {
        task_id: "t".to_owned(),
        field: "f".to_owned(),
        message: "m".to_owned(),
        location: mindtape::Location::at("tasks/t.typ", line),
        severity,
        kind: mindtape::ViolationKind::Other,
        reads: None,
        unknown: None,
    }
}

#[test]
fn every_breach_line_starts_with_a_severity() {
    use mindtape::Severity::{Error, Warn};
    let show = |s, l| crate::render::breach::line(&breach(s, l));
    assert_eq!(show(Error, 3), "error: tasks/t.typ:3: f: m");
    assert_eq!(show(Warn, 3), "warning: tasks/t.typ:3: f: m");
}

#[test]
fn a_built_in_template_breach_names_no_path() {
    let mut builtin = breach(mindtape::Severity::Warn, 3);
    builtin.location = mindtape::Location::Builtin {
        template: "task".to_owned(),
        line: 3,
    };
    assert_eq!(
        crate::render::breach::line(&builtin),
        "warning: built-in template task:3: f: m"
    );
}
