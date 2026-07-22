//! Tests for the `[[on.flip]]` command executor: capture, verdict, context.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::{ENV_FILE, ENV_STATUS, ENV_STEM, RunCtx, run_command};

fn ctx<'a>(root: &'a Path, path: &'a Path) -> RunCtx<'a> {
    RunCtx {
        root,
        path,
        stem: "alpha",
        target: "done",
    }
}

#[test]
fn a_passing_command_reports_success_and_its_output() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tasks/alpha.typ");
    let report = run_command("echo hello", &ctx(dir.path(), &path)).unwrap();
    assert!(report.success);
    assert_eq!(report.code, Some(0));
    assert_eq!(report.output, "hello\n");
}

/// Both streams are captured: a gate's evidence is usually on stderr.
#[test]
fn stdout_and_stderr_are_both_captured() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tasks/alpha.typ");
    let report = run_command("echo out; echo err >&2; exit 4", &ctx(dir.path(), &path)).unwrap();
    assert!(!report.success);
    assert_eq!(report.code, Some(4));
    assert!(report.output.contains("out"), "{}", report.output);
    assert!(report.output.contains("err"), "{}", report.output);
    assert_eq!(report.verdict(), "exit 4");
}

/// A signal leaves no exit code, and the verdict says so rather than inventing
/// one — a gate killed by a timeout must not read as a clean failure.
#[test]
fn a_signalled_command_has_no_code() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tasks/alpha.typ");
    let report = run_command("kill -9 $$", &ctx(dir.path(), &path)).unwrap();
    assert!(!report.success);
    assert_eq!(report.code, None);
    assert_eq!(report.verdict(), "terminated by signal");
}

/// The command runs in the project root, with the transition in its
/// environment — the contract a gate script is written against.
#[test]
fn the_command_sees_the_root_and_the_transition() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("marker"), "x").unwrap();
    let path = dir.path().join("tasks/alpha.typ");
    let script = format!("test -f marker && echo \"${ENV_STEM}|${ENV_STATUS}|${ENV_FILE}\"");
    let report = run_command(&script, &ctx(dir.path(), &path)).unwrap();
    assert!(report.success, "{}", report.output);
    assert_eq!(
        report.output.trim(),
        format!("alpha|done|{}", path.display())
    );
}
