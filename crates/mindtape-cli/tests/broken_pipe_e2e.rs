//! End-to-end tests for what `mt` does when its reader walks away
//! (`truth/tasks/mt-exits-141-on-sigpipe-about-one-call-in-ten.typ`).
//!
//! Staged the only way a broken pipe can be staged honestly: spawn `mt` with a
//! piped stdout and close the read end before it writes. Redirecting to a
//! closed file descriptor or piping into `head` both leave the close racing the
//! write, which is how the failure this pins stayed a "one call in ten" for
//! weeks instead of a reproducer.
//!
//! What is asserted is the pair the task named. `mt` must not die of a signal
//! and must not exit 141 — the report's symptom, which turned out to belong to
//! a shell wrapper on `PATH` rather than to this binary, and is asserted here
//! so the binary's side of it can never quietly become true. And it must not
//! panic, which before `crate::stdout` it always did: libstd ignores `SIGPIPE`,
//! so the write returned `EPIPE` and the prelude's `println!` unwrapped it into
//! exit 101 and a `failed printing to stdout` backtrace.
#![allow(clippy::unwrap_used)]

use std::io::Read;
use std::process::{Command, Stdio};

mod support;

/// Run `mt <args>` in `dir` with stdout piped, drop the read end immediately,
/// and return the finished process. Everything the command would have printed
/// goes into a pipe with no reader, so the first write fails.
fn run_with_closed_reader(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mt"));
    command.current_dir(dir).args(args);
    support::isolate(&mut command, dir);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Closing the read end is the whole fixture: from here every write `mt`
    // makes to stdout is EPIPE. Taking the handle out of the child and dropping
    // it is what closes it.
    drop(child.stdout.take());
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    let status = child.wait().unwrap();
    std::process::Output {
        status,
        stdout: Vec::new(),
        stderr: stderr.into_bytes(),
    }
}

/// Assert `output` ended the way a command whose reader left should end: on its
/// own terms, quietly.
fn assert_clean_exit(output: &std::process::Output, what: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(
        output.status.code(),
        Some(141),
        "{what}: exited 141 (the reported SIGPIPE symptom); stderr: {stderr}"
    );
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(
            output.status.signal(),
            None,
            "{what}: killed by a signal rather than exiting; stderr: {stderr}"
        );
    }
    assert!(
        !stderr.contains("panicked"),
        "{what}: panicked on a closed reader; stderr: {stderr}"
    );
    assert!(
        output.status.success(),
        "{what}: a reader that stopped reading is not this command's failure; \
         status {:?}, stderr: {stderr}",
        output.status.code()
    );
}

#[test]
fn ls_survives_a_reader_that_closed_the_pipe() {
    let dir = support::project_with_tasks(&[("a.typ", "Alpha"), ("b.typ", "Beta")]);
    let output = run_with_closed_reader(dir.path(), &["ls", ""]);
    assert_clean_exit(&output, "mt ls");
}

/// The JSON path prints one large `serde_json` string rather than the table
/// renderer's, so it exercises a different write site with a different shape.
#[test]
fn ls_json_survives_a_reader_that_closed_the_pipe() {
    let dir = support::project_with_tasks(&[("a.typ", "Alpha"), ("b.typ", "Beta")]);
    let output = run_with_closed_reader(dir.path(), &["ls", "", "--json"]);
    assert_clean_exit(&output, "mt ls --json");
}

/// A mutating command is the case the policy was chosen for: dropping the write
/// must not drop the work, so the task is on disk afterwards and `mt` says so
/// with a zero status rather than aborting half way through.
#[test]
fn add_still_files_the_task_when_the_reader_is_gone() {
    let dir = support::scaffold_project("[tasks]\nroots = [\"tasks\"]\n");
    let output = run_with_closed_reader(
        dir.path(),
        &["add", "--name", "quiet-reader", "--set", "title=T"],
    );
    assert_clean_exit(&output, "mt add");
    assert!(
        dir.path().join("tasks/quiet-reader.typ").is_file(),
        "the file the command exists to write must survive its output being dropped"
    );
}
