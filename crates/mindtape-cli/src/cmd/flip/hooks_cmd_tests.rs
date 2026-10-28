//! Tests for the `[[on.flip]]` command executor: capture, verdict, context.
//!
//! The cases below are shell lines, written in the shell
//! `mindtape::shell::SHELL` names and split by platform: the POSIX set carries the full
//! contract (including the signal verdict, which Windows has no analogue for),
//! and a smaller `cmd` set pins the same capture-and-context behaviour there.
//! An array `run` needs no shell, so `tests/it/argv_run_e2e.rs` covers it on
//! every platform at once.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::{tempdir, tempdir_with_files};
use std::path::{Path, PathBuf};

use mindtape::shell::ConfigCommand;

use super::{CommandReport, ENV_FILE, ENV_STATUS, ENV_STEM, RunCtx, run_command};

/// `text` as a shell line, the form every case below is written in.
fn line(text: &str) -> ConfigCommand {
    ConfigCommand::Line(text.to_owned())
}

fn ctx<'a>(root: &'a Path, path: &'a Path) -> RunCtx<'a> {
    RunCtx {
        root,
        path,
        stem: "alpha",
        target: "done",
    }
}

/// Run the shell line `text` from the project root `root`, flipping the task
/// `tasks/alpha.typ`, and return the report and that task's path.
fn run_in(root: &Path, text: &str) -> (CommandReport, PathBuf) {
    let path = root.join("tasks/alpha.typ");
    let report = run_command(&line(text), &ctx(root, &path)).unwrap();
    (report, path)
}

/// [`run_in`] an empty project root.
fn run_line(text: &str) -> CommandReport {
    let dir = tempdir();
    run_in(dir.path(), text).0
}

/// A shell that cannot start is an error, not a failed command: a missing
/// project root leaves the spawn nothing to run in, so the caller must not read
/// the absence of a verdict as a pass.
#[test]
fn a_command_that_cannot_spawn_is_an_error_naming_it() {
    let dir = tempdir();
    let root = dir.path().join("no-such-root");
    let path = root.join("tasks/alpha.typ");
    let err = match run_command(&line("echo hi"), &ctx(&root, &path)) {
        Err(err) => crate::diag::plain(&err),
        Ok(report) => panic!("expected a spawn error, got {}", report.output),
    };
    assert!(
        err.starts_with("error: can't run on.flip command: 'echo hi'"),
        "{err}"
    );
}

#[cfg(not(windows))]
#[test]
fn a_passing_command_reports_success_and_its_output() {
    let report = run_line("echo hello");
    assert!(report.success);
    assert_eq!(report.code, Some(0));
    assert_eq!(report.output, "hello\n");
}

/// Both streams are captured: a gate's evidence is usually on stderr.
#[cfg(not(windows))]
#[test]
fn stdout_and_stderr_are_both_captured() {
    let report = run_line("echo out; echo err >&2; exit 4");
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
    let report = run_line("kill -9 $$");
    assert!(!report.success);
    assert_eq!(report.code, None);
    assert_eq!(report.verdict(), "terminated by signal");
}

/// The command runs in the project root, with the transition in its
/// environment — the contract a gate script is written against.
#[cfg(not(windows))]
#[test]
fn the_command_sees_the_root_and_the_transition() {
    let dir = tempdir_with_files(&[("marker", "x")]);
    let script = format!("test -f marker && echo \"${ENV_STEM}|${ENV_STATUS}|${ENV_FILE}\"");
    let (report, path) = run_in(dir.path(), &script);
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
    use super::{ENV_FILE, ENV_STATUS, ENV_STEM, run_in, run_line, tempdir_with_files};

    #[test]
    fn a_passing_command_reports_success_and_its_output() {
        let report = run_line("echo hello");
        assert!(report.success, "{}", report.output);
        assert_eq!(report.code, Some(0));
        assert_eq!(report.output.trim(), "hello");
    }

    /// A non-zero exit is reported, not raised — the caller decides about it.
    #[test]
    fn a_failing_command_reports_its_code() {
        let report = run_line("exit /b 4");
        assert!(!report.success);
        assert_eq!(report.code, Some(4));
        assert_eq!(report.verdict(), "exit 4");
    }

    /// Both streams are captured: a gate's evidence is usually on stderr.
    #[test]
    fn stdout_and_stderr_are_both_captured() {
        let report = run_line("echo out& echo err 1>&2");
        assert!(report.output.contains("out"), "{}", report.output);
        assert!(report.output.contains("err"), "{}", report.output);
    }

    /// The command runs in the project root, with the transition in its
    /// environment.
    #[test]
    fn the_command_sees_the_root_and_the_transition() {
        let dir = tempdir_with_files(&[("marker", "x")]);
        let script = format!("if exist marker echo %{ENV_STEM}% %{ENV_STATUS}% %{ENV_FILE}%");
        let (report, _) = run_in(dir.path(), &script);
        assert!(report.success, "{}", report.output);
        let out = report.output.trim();
        assert!(out.starts_with("alpha done"), "{out}");
        assert!(out.contains("alpha.typ"), "{out}");
    }
}
