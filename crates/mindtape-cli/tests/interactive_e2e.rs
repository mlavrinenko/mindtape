//! End-to-end tests for `mt ls --interactive`.
//!
//! Spawns the binary under `script` (which provides a pseudo-terminal) so the
//! TTY checks in `guard` pass, then sends `q` to quit and asserts that the task
//! table was rendered. Catches the body-replacement mutants on `event_loop` and
//! `draw`, and every mutant in `guard` — including `||` -> `&&`, which only the
//! piped-stdin case below can tell apart.
#![allow(clippy::unwrap_used)]

mod support;

use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// Interval between `try_wait()` polls while waiting for `mt` to exit.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Upper bound on how long `mt` may take to exit after receiving `q`. The
/// happy path finishes in well under a second; this only needs to be long
/// enough to absorb slow CI machines while still failing fast (instead of
/// hanging the whole test binary forever) if `event_loop` never returns. It is
/// what makes `event_loop`'s `!=` -> `==` mutant killable: that mutant drops
/// every `Press` event, so the loop never sees the `q` and this test fails on
/// the deadline instead of hanging the sweep.
const WAIT_TIMEOUT: Duration = Duration::from_secs(5);

/// Wait for `child` to exit, polling instead of blocking so a hung event loop
/// fails this test instead of hanging the whole suite. Kills the process and
/// returns its (partial) output plus `true` if `WAIT_TIMEOUT` elapses first.
fn wait_with_timeout(mut child: Child) -> (Output, bool) {
    let deadline = Instant::now() + WAIT_TIMEOUT;
    let mut timed_out = false;
    loop {
        match child.try_wait().expect("failed to poll mt process status") {
            Some(_status) => break,
            None if Instant::now() >= deadline => {
                timed_out = true;
                break;
            }
            None => std::thread::sleep(POLL_INTERVAL),
        }
    }
    if timed_out {
        // Not exited by the deadline: kill it so wait_with_output below
        // can't block forever collecting its output.
        let _ = child.kill();
    }
    (child.wait_with_output().unwrap(), timed_out)
}

/// The title of the one task each case lists.
const TASK_TITLE: &str = "E2E Task";

/// Spawn `mt ls --interactive <suffix>` inside `dir` under `script`, which
/// hands it a pseudo-terminal. Stdin is piped so the caller can send keys.
fn spawn_interactive(dir: &std::path::Path, suffix: &str) -> Child {
    let mt_bin = PathBuf::from(env!("CARGO_BIN_EXE_mt"));
    let mt_str = mt_bin.to_string_lossy().to_string();
    Command::new("script")
        .args([
            "-q",
            "-e",
            "-c",
            &format!("{mt_str} ls --interactive{suffix}"),
            "/dev/null",
        ])
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("script must be available (util-linux)")
}

#[test]
fn interactive_mode_renders_tasks_and_quits() {
    let dir = support::project_with_task(TASK_TITLE);
    let mut child = spawn_interactive(dir.path(), "");

    // Give mt a moment to start the event loop and draw the table.
    std::thread::sleep(Duration::from_millis(300));

    // Send 'q' to quit interactive mode.
    child
        .stdin
        .take()
        .expect("stdin pipe must be available")
        .write_all(b"q\n")
        .unwrap();

    let (output, timed_out) = wait_with_timeout(child);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !timed_out,
        "mt did not exit within {WAIT_TIMEOUT:?} after sending 'q' -- event_loop appears hung\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        stdout.contains(TASK_TITLE),
        "stdout should contain the rendered task table\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        output.status.success(),
        "mt should exit successfully (status: {:?})",
        output.status.code(),
    );
}

/// `guard` requires a TTY on *both* stdin and stdout. Redirecting only stdin
/// from `/dev/null` leaves stdout on the pty, so exactly one half of the check
/// fails — the only shape that tells `!stdin || !stdout` apart from `&&`.
#[test]
fn interactive_refuses_piped_stdin_when_stdout_is_a_tty() {
    let dir = support::project_with_task(TASK_TITLE);
    let mut child = spawn_interactive(dir.path(), " < /dev/null");

    // crossterm reads keys from /dev/tty, not stdin, so a `&&` guard would
    // enter the event loop regardless of the redirect. Send 'q' so that case
    // quits instead of hanging until the timeout.
    child
        .stdin
        .take()
        .expect("stdin pipe must be available")
        .write_all(b"q\n")
        .unwrap();

    let (output, timed_out) = wait_with_timeout(child);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !timed_out,
        "mt should refuse immediately\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("needs a TTY on stdin and stdout"),
        "piped stdin must be rejected by name\nstdout:\n{stdout}"
    );
    assert!(
        !output.status.success(),
        "mt should exit non-zero (status: {:?})",
        output.status.code(),
    );
}

// Terminal control sequences crossterm emits for the alternate screen and the
// cursor. Asserting on the bytes is the point: under a pty they are really
// written, which is what makes the teardown observable at all.
const ENTER_ALT: &str = "\x1b[?1049h";
const LEAVE_ALT: &str = "\x1b[?1049l";
const CURSOR_HIDE: &str = "\x1b[?25l";
const CURSOR_SHOW: &str = "\x1b[?25h";

/// Quitting must restore the terminal: leave the alternate screen and show the
/// cursor again, after having entered and hidden them.
///
/// `.cargo/mutants.toml` used to excuse `Tui::drop` and `leave_screen` as
/// unobservable, on the reasoning that `leave_screen` errors (or is a no-op)
/// without a TTY. Under this pty there *is* a TTY, so both write real bytes
/// into the captured stream: an empty `Drop` body or a `leave_screen` replaced
/// by `Ok(())` leaves the transcript without `LEAVE_ALT`/`CURSOR_SHOW` and
/// fails here.
#[test]
fn quitting_restores_the_terminal() {
    let dir = support::project_with_task(TASK_TITLE);
    let mut child = spawn_interactive(dir.path(), "");

    std::thread::sleep(Duration::from_millis(300));
    child
        .stdin
        .take()
        .expect("stdin pipe must be available")
        .write_all(b"q\n")
        .unwrap();

    let (output, timed_out) = wait_with_timeout(child);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!timed_out, "mt did not exit within {WAIT_TIMEOUT:?}");

    let enter = stdout
        .find(ENTER_ALT)
        .expect("must enter the alternate screen");
    let leave = stdout
        .rfind(LEAVE_ALT)
        .expect("teardown must leave the alternate screen");
    assert!(
        leave > enter,
        "the alternate screen must be left after it was entered ({enter} -> {leave})"
    );

    let hide = stdout.find(CURSOR_HIDE).expect("must hide the cursor");
    let show = stdout
        .rfind(CURSOR_SHOW)
        .expect("teardown must show it again");
    assert!(
        show > hide,
        "the cursor must be shown again after being hidden ({hide} -> {show})"
    );
}

/// Opening a task suspends the TUI around the editor spawn: `suspend` leaves
/// the alternate screen, runs the closure, and re-enters. `$EDITOR` here just
/// echoes a marker, so the marker appearing in the transcript proves the
/// closure ran — which is exactly what `replace suspend -> Result<()> with
/// Ok(())` would skip.
///
/// This replaces the former in-process `suspend_executes_closure`, which built
/// a `Tui` by hand and drove the developer's own terminal into raw mode.
#[test]
fn opening_a_task_suspends_the_tui_and_runs_the_editor() {
    let stdout = select_first_task_and_open(&[]);
    assert!(
        stdout.contains("SUSPEND_MARKER"),
        "the editor closure must run inside suspend\nstdout:\n{stdout}"
    );
}

/// Drive one interactive session under a pseudo-terminal: select the first
/// task, press Enter to open it, then quit. `extra_args` is appended to
/// `ls --interactive` (e.g. `["--hyperlinks", "always"]`). Returns the
/// session's stdout, having already asserted it exited within
/// [`WAIT_TIMEOUT`].
///
/// Shared rather than inlined per test: the spawn-and-drive sequence is
/// twenty lines of pty plumbing that says nothing about what any individual
/// test is asserting, and two copies of it drifting apart is exactly the bug
/// these tests exist to catch.
fn select_first_task_and_open(extra_args: &[&str]) -> String {
    let dir = support::project_with_task(TASK_TITLE);
    let mt_bin = PathBuf::from(env!("CARGO_BIN_EXE_mt"));
    let mut command = format!("{} ls --interactive", mt_bin.to_string_lossy());
    for arg in extra_args {
        command.push(' ');
        command.push_str(arg);
    }
    let mut child = Command::new("script")
        .args(["-q", "-e", "-c", &command, "/dev/null"])
        .current_dir(dir.path())
        // `resolve_editor` prefers $VISUAL over $EDITOR, so an inherited
        // $VISUAL (a real editor, often with --wait) would hang this test on
        // the developer's machine while passing in a bare CI environment.
        .env_remove("VISUAL")
        .env("EDITOR", "echo SUSPEND_MARKER")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("script must be available (util-linux)");

    let mut stdin = child.stdin.take().expect("stdin pipe must be available");
    std::thread::sleep(Duration::from_millis(300));
    // Select the first task, then Enter (CR in raw mode) to open it.
    stdin.write_all(b"1\r").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    stdin.write_all(b"q\n").unwrap();
    drop(stdin);

    let (output, timed_out) = wait_with_timeout(child);
    assert!(!timed_out, "mt did not exit within {WAIT_TIMEOUT:?}");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The risk `truth/tasks/hyperlink-the-handle-column.typ` exists to guard:
/// the `#` column's cells now carry an OSC 8 wrap (and, once a row is typed,
/// an invert-colour wrap on top of that), but the key loop reads real key
/// events off the terminal (crossterm), never the rendered screen — so typing
/// a digit and Enter must still select and open the right task with
/// hyperlinks active. Mirrors
/// `opening_a_task_suspends_the_tui_and_runs_the_editor`, with
/// `--hyperlinks always` added.
#[test]
fn opening_a_task_still_works_with_hyperlinks_active() {
    let stdout = select_first_task_and_open(&["--hyperlinks", "always"]);
    assert!(
        stdout.contains("\u{1b}]8;;file://"),
        "the # column must actually carry the OSC 8 wrap in this run\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("SUSPEND_MARKER"),
        "selecting task 1 and pressing Enter must still open the editor with \
         hyperlinks active\nstdout:\n{stdout}"
    );
}
