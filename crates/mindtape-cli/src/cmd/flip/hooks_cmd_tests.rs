//! Tests for the `[[on.flip]]` command executor: capture, verdict, context.
//!
//! A `run` command is shell text, so the cases below are written in the shell
//! `mindtape::shell::SHELL` names and split by platform: the POSIX set carries the full
//! contract (including the signal verdict, which Windows has no analogue for),
//! and a smaller `cmd` set pins the same capture-and-context behaviour there.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir;
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

#[cfg(not(windows))]
#[test]
fn a_passing_command_reports_success_and_its_output() {
    let dir = tempdir();
    let path = dir.path().join("tasks/alpha.typ");
    let report = run_command("echo hello", &ctx(dir.path(), &path)).unwrap();
    assert!(report.success);
    assert_eq!(report.code, Some(0));
    assert_eq!(report.output, "hello\n");
}

/// Both streams are captured: a gate's evidence is usually on stderr.
#[cfg(not(windows))]
#[test]
fn stdout_and_stderr_are_both_captured() {
    let dir = tempdir();
    let path = dir.path().join("tasks/alpha.typ");
    let report = run_command("echo out; echo err >&2; exit 4", &ctx(dir.path(), &path)).unwrap();
    assert!(!report.success);
    assert_eq!(report.code, Some(4));
    assert!(report.output.contains("out"), "{}", report.output);
    assert!(report.output.contains("err"), "{}", report.output);
    assert_eq!(report.verdict(), "exit 4");
}

/// A signal leaves no exit code, and the verdict says so rather than inventing
/// one — a gate killed by a timeout must not read as a clean failure. Unix
/// only: a Windows process always ends with an exit code, so there is no state
/// to reach `verdict`'s `None` arm with.
#[cfg(unix)]
#[test]
fn a_signalled_command_has_no_code() {
    let dir = tempdir();
    let path = dir.path().join("tasks/alpha.typ");
    let report = run_command("kill -9 $$", &ctx(dir.path(), &path)).unwrap();
    assert!(!report.success);
    assert_eq!(report.code, None);
    assert_eq!(report.verdict(), "terminated by signal");
}

/// The command runs in the project root, with the transition in its
/// environment — the contract a gate script is written against.
#[cfg(not(windows))]
#[test]
fn the_command_sees_the_root_and_the_transition() {
    let dir = tempdir();
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

/// The same executor contract under `cmd /C`. Deliberately thinner than the
/// POSIX set above and asserted with `contains` rather than equality: `cmd`
/// echoes CRLF and its own quoting rules, and the behaviour under test is the
/// capture, the code, and the context — not the shell's formatting.
#[cfg(windows)]
mod windows {
    use super::{ENV_FILE, ENV_STATUS, ENV_STEM, ctx, run_command};
    use mindtape::test_support::tempdir;

    #[test]
    fn a_passing_command_reports_success_and_its_output() {
        let dir = tempdir();
        let path = dir.path().join("tasks/alpha.typ");
        let report = run_command("echo hello", &ctx(dir.path(), &path)).unwrap();
        assert!(report.success, "{}", report.output);
        assert_eq!(report.code, Some(0));
        assert_eq!(report.output.trim(), "hello");
    }

    /// A non-zero exit is reported, not raised — the caller decides about it.
    #[test]
    fn a_failing_command_reports_its_code() {
        let dir = tempdir();
        let path = dir.path().join("tasks/alpha.typ");
        let report = run_command("exit /b 4", &ctx(dir.path(), &path)).unwrap();
        assert!(!report.success);
        assert_eq!(report.code, Some(4));
        assert_eq!(report.verdict(), "exit 4");
    }

    /// Both streams are captured: a gate's evidence is usually on stderr.
    #[test]
    fn stdout_and_stderr_are_both_captured() {
        let dir = tempdir();
        let path = dir.path().join("tasks/alpha.typ");
        let report = run_command("echo out& echo err 1>&2", &ctx(dir.path(), &path)).unwrap();
        assert!(report.output.contains("out"), "{}", report.output);
        assert!(report.output.contains("err"), "{}", report.output);
    }

    /// The command runs in the project root, with the transition in its
    /// environment.
    #[test]
    fn the_command_sees_the_root_and_the_transition() {
        let dir = tempdir();
        std::fs::write(dir.path().join("marker"), "x").unwrap();
        let path = dir.path().join("tasks/alpha.typ");
        let script = format!("if exist marker echo %{ENV_STEM}% %{ENV_STATUS}% %{ENV_FILE}%");
        let report = run_command(&script, &ctx(dir.path(), &path)).unwrap();
        assert!(report.success, "{}", report.output);
        let out = report.output.trim();
        assert!(out.starts_with("alpha done"), "{out}");
        assert!(out.contains("alpha.typ"), "{out}");
    }
}
