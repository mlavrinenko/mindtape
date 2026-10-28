//! End-to-end tests for `mt ls --interactive`.
//!
//! Spawns the binary under `script` (which provides a pseudo-terminal) so the
//! TTY checks in `guard` pass, then sends Esc to quit and asserts that the task
//! table was rendered. Catches the body-replacement mutants on `event_loop` and
//! `draw`, and every mutant in `guard` — including `||` -> `&&`, which only the
//! piped-stdin case below can tell apart.
//!
//! Unix only, for the same reason as `tty_e2e.rs`: the pty comes from
//! util-linux `script`, which Windows has no equivalent of.
#![cfg(unix)]
#![allow(clippy::unwrap_used)]

use crate::support;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdout, Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Interval between `try_wait()` polls while waiting for `mt` to exit.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Upper bound on how long `mt` may take to exit after receiving Esc. The
/// happy path finishes in well under a second; this only needs to be long
/// enough to absorb a machine busy with other builds while still failing
/// (instead of hanging the whole test binary forever) if `event_loop` never
/// returns. It is a backstop only: the tests send Esc once the TUI has shown
/// it is reading again, so load makes the exit slower, not lost. It is
/// what makes `event_loop`'s `!=` -> `==` mutant killable: that mutant drops
/// every `Press` event, so the loop never sees the Esc and this test fails on
/// the deadline instead of hanging the sweep.
const WAIT_TIMEOUT: Duration = Duration::from_secs(30);

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

/// Esc on its own, so crossterm reads it as the key rather than as the start
/// of an Alt chord: the picker's quit, since every letter is a hash digit.
const ESC: &[u8] = b"\x1b";

/// The title of the one task each case lists.
const TASK_TITLE: &str = "E2E Task";

/// What the stand-in `$EDITOR` prints, so its appearance in the transcript
/// proves the editor closure actually ran. Named once and reused by both the
/// spawn (which tells `$EDITOR` to print it) and the tests that poll for it
/// ([`wait_for_frame`]), rather than repeating the literal on both sides.
const SUSPEND_MARKER: &str = "SUSPEND_MARKER";

/// Spawn `mt ls --interactive <suffix>` inside `dir` under `script`, which
/// hands it a pseudo-terminal. Stdin is piped so the caller can send keys.
/// `with_editor` sets `$EDITOR` to echo [`SUSPEND_MARKER`] (and clears
/// `$VISUAL`, which would otherwise inherit a real editor and hang the
/// session) — the shape the opening tests need.
fn spawn_interactive(dir: &std::path::Path, suffix: &str, with_editor: bool) -> Child {
    let mt_bin = PathBuf::from(env!("CARGO_BIN_EXE_mt"));
    let mt_str = mt_bin.to_string_lossy().to_string();
    let mut script = Command::new("script");
    let mut cmd = support::isolate(&mut script, dir)
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
        .stderr(Stdio::piped());
    if with_editor {
        cmd = cmd
            .env_remove("VISUAL")
            .env("EDITOR", format!("echo {SUSPEND_MARKER}"));
    }
    cmd.spawn().expect("script must be available (util-linux)")
}

/// The pty transcript captured so far, shared between the reader thread
/// draining `child.stdout` in [`stream_into`] and [`wait_for_frame`] polling
/// it — a `String` rather than raw bytes because every needle the tests poll
/// for is text.
type Frame = Arc<Mutex<String>>;

/// Drain `stdout` into `frame` on a background thread, appending each chunk
/// as it arrives. Returns the thread's handle so a caller can join it once
/// the child has exited, guaranteeing `frame` holds everything the process
/// ever wrote rather than whatever had arrived by the last poll.
///
/// Reading incrementally (rather than the once-at-exit `wait_with_output`
/// every other test in this file uses) is what makes [`wait_for_frame`]
/// possible at all: polling needs a transcript that grows while the child is
/// still running, not one only available after it quits.
fn stream_into(mut stdout: ChildStdout, frame: Frame) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buf = [0_u8; 4096];
        loop {
            match stdout.read(&mut buf) {
                Ok(0) | Err(_) => return,
                Ok(n) => {
                    let Some(chunk) = buf.get(..n) else {
                        return;
                    };
                    let mut guard = frame.lock().expect("frame mutex must not be poisoned");
                    guard.push_str(&String::from_utf8_lossy(chunk));
                }
            }
        }
    })
}

/// Upper bound on how long [`wait_for_frame`] polls for a needle before
/// giving up. Generous — this only needs to absorb a machine busy with other
/// test suites, never the happy path, which reaches every awaited frame in
/// well under a second — because the failure this bounds (a hung or broken
/// event loop) should still fail the test rather than hang the sweep.
const FRAME_TIMEOUT: Duration = Duration::from_secs(15);

/// Interval between polls in [`wait_for_frame`].
const FRAME_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Block until `frame` contains `needle` at or after byte offset `from`,
/// polling instead of sleeping a fixed span so a busy machine only makes this
/// slower, never flaky — the fix for the two tests
/// `truth/tasks/isolate-the-interactive-pty-tests-from-concurrent-load.typ`
/// describes, which raced a fixed wait against however long the TUI actually
/// took to reach the state being asserted. Returns the offset just past the
/// match, so a caller can wait for a later signal in order. Panics naming the
/// last frame actually seen if [`FRAME_TIMEOUT`] elapses first.
fn wait_for_frame_from(frame: &Frame, needle: &str, from: usize) -> usize {
    let deadline = Instant::now() + FRAME_TIMEOUT;
    loop {
        let snapshot = frame
            .lock()
            .expect("frame mutex must not be poisoned")
            .clone();
        if let Some(at) = snapshot.get(from..).and_then(|tail| tail.find(needle)) {
            return from + at + needle.len();
        }
        assert!(
            Instant::now() < deadline,
            "timed out after {FRAME_TIMEOUT:?} waiting for {needle:?}\nlast frame seen:\n{snapshot}"
        );
        std::thread::sleep(FRAME_POLL_INTERVAL);
    }
}

/// [`wait_for_frame_from`] anywhere in the transcript.
fn wait_for_frame(frame: &Frame, needle: &str) {
    wait_for_frame_from(frame, needle, 0);
}

/// Drive one interactive session under a pseudo-terminal: send `keys` (already
/// shaped for raw mode), wait for the frames `expect_after_keys` names (if
/// any), each after the previous, to actually appear, then Esc to quit, and assert a clean exit within
/// [`WAIT_TIMEOUT`]. Returns the session's full transcript.
///
/// Shared rather than inlined per test: the spawn-and-drive sequence is
/// twenty lines of pty plumbing that says nothing about what any individual
/// test is asserting, and two copies of it drifting apart is exactly the bug
/// these tests exist to catch.
///
/// The wait before `keys` is always a poll for the rendered task table
/// ([`TASK_TITLE`]), never a fixed sleep: `keys` sent before the event loop is
/// actually reading them is exactly the race
/// `truth/tasks/isolate-the-interactive-pty-tests-from-concurrent-load.typ`
/// files. The wait after `keys` polls too when the caller has something to
/// wait for; a caller with nothing to observe (a keystroke that should open
/// nothing) still gives the loop a moment to settle before quitting.
fn drive_session(
    dir: &std::path::Path,
    suffix: &str,
    keys: &[u8],
    expect_after_keys: &[&str],
) -> String {
    let mut child = spawn_interactive(dir, suffix, true);
    let stdout = child.stdout.take().expect("stdout pipe must be available");
    let frame: Frame = Arc::new(Mutex::new(String::new()));
    let reader = stream_into(stdout, Arc::clone(&frame));

    let mut stdin = child.stdin.take().expect("stdin pipe must be available");
    wait_for_frame(&frame, TASK_TITLE);
    stdin.write_all(keys).unwrap();
    if expect_after_keys.is_empty() {
        std::thread::sleep(Duration::from_millis(400));
    }
    let mut seen = 0;
    for needle in expect_after_keys {
        seen = wait_for_frame_from(&frame, needle, seen);
    }
    stdin.write_all(ESC).unwrap();
    drop(stdin);

    let (output, timed_out) = wait_with_timeout(child);
    assert!(!timed_out, "mt did not exit within {WAIT_TIMEOUT:?}");
    assert!(
        output.status.success(),
        "mt should exit successfully (status: {:?})",
        output.status.code(),
    );
    reader.join().expect("stdout reader thread must not panic");
    Arc::try_unwrap(frame)
        .expect("no other holder of frame remains once the reader thread has joined")
        .into_inner()
        .expect("frame mutex must not be poisoned")
}

#[test]
fn interactive_mode_renders_tasks_and_quits() {
    let dir = support::project_with_task(TASK_TITLE);

    // Space reloads the view (the Refresh action); `drive_session` waits for
    // the drawn table before it sends the key and Esc, and asserts a clean exit.
    let stdout = drive_session(dir.path(), "", b" ", &[TASK_TITLE]);

    assert!(
        stdout.contains(TASK_TITLE),
        "stdout should contain the rendered task table\nstdout:\n{stdout}"
    );
}

/// `guard` requires a TTY on *both* stdin and stdout. Redirecting only stdin
/// from `/dev/null` leaves stdout on the pty, so exactly one half of the check
/// fails — the only shape that tells `!stdin || !stdout` apart from `&&`.
#[test]
fn interactive_refuses_piped_stdin_when_stdout_is_a_tty() {
    let dir = support::project_with_task(TASK_TITLE);
    let mut child = spawn_interactive(dir.path(), " < /dev/null", false);

    // crossterm reads keys from /dev/tty, not stdin, so a `&&` guard would
    // enter the event loop regardless of the redirect. Send Esc so that case
    // quits instead of hanging until the timeout.
    child
        .stdin
        .take()
        .expect("stdin pipe must be available")
        .write_all(ESC)
        .unwrap();

    let (output, timed_out) = wait_with_timeout(child);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !timed_out,
        "mt should refuse immediately\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("needs a terminal on stdin and stdout"),
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
/// `.config/mutants.toml` used to excuse `Tui::drop` and `leave_screen` as
/// unobservable, on the reasoning that `leave_screen` errors (or is a no-op)
/// without a TTY. Under this pty there *is* a TTY, so both write real bytes
/// into the captured stream: an empty `Drop` body or a `leave_screen` replaced
/// by `Ok(())` leaves the transcript without `LEAVE_ALT`/`CURSOR_SHOW` and
/// fails here.
#[test]
fn quitting_restores_the_terminal() {
    let dir = support::project_with_task(TASK_TITLE);

    // Esc goes out only once the table is drawn, so the event loop is reading.
    let stdout = drive_session(dir.path(), "", b"", &[TASK_TITLE]);

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
    let stdout = select_first_task_and_open("");
    assert!(
        stdout.contains(SUSPEND_MARKER),
        "the editor closure must run inside suspend\nstdout:\n{stdout}"
    );
}

/// Enter with no task selected must not spawn the editor: `collect_first`
/// returns nothing for a buffer that matches no stem hash, `open` no-ops on an
/// empty index set, and the loop keeps running. The transcript has no
/// [`SUSPEND_MARKER`] and the session still quits cleanly on Esc.
#[test]
fn enter_without_a_match_opens_nothing() {
    let dir = support::project_with_task(TASK_TITLE);
    // "u" is no hash digit, so it matches no row, then Enter. Nothing should
    // ever open, so there is no frame to poll for here.
    let stdout = drive_session(dir.path(), "", b"u\r", &[]);
    assert!(
        !stdout.contains(SUSPEND_MARKER),
        "a no-match Enter must not spawn the editor\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains(TASK_TITLE),
        "the table must still be rendered\nstdout:\n{stdout}"
    );
}

/// Drive one interactive session under a pseudo-terminal: select the first
/// task, press Enter to open it, wait for the editor to actually run, then
/// quit. `suffix` is appended to `ls --interactive` (e.g.
/// ` --hyperlinks always`). Returns the session's stdout, having already
/// asserted it exited cleanly within [`WAIT_TIMEOUT`].
fn select_first_task_and_open(suffix: &str) -> String {
    open_first_task_in(&support::project_with_task(TASK_TITLE), suffix)
}

/// [`select_first_task_and_open`] in the already-built project `dir`.
fn open_first_task_in(dir: &tempfile::TempDir, suffix: &str) -> String {
    // Type the task's stem hash, then Enter (CR in raw mode) to open it; wait
    // for SUSPEND_MARKER rather than guessing how long the editor takes to run.
    let hash = mindtape::StemHashes::new(["test"])
        .hash_of("test")
        .unwrap()
        .to_owned();
    let keys = format!("{hash}\r");
    drive_session(
        dir.path(),
        suffix,
        keys.as_bytes(),
        // The marker proves the editor ran; the alternate screen entered after
        // it proves `suspend` has re-entered the TUI, so Esc reaches a loop
        // that is reading rather than one still switching terminal modes.
        &[SUSPEND_MARKER, ENTER_ALT],
    )
}

/// A project `[go] editor` waits for `mt allow` in the picker's open too: the
/// held editor never runs, `$EDITOR` opens the task, and the warning after the
/// session names the held command.
#[test]
fn opening_a_task_holds_a_project_editor_and_falls_back_to_editor() {
    let dir = support::project_with_task(TASK_TITLE);
    mindtape::test_support::write_file(dir.path(), "held.sh", "touch held-ran\n");
    let command = format!("sh {}", dir.path().join("held.sh").to_string_lossy());
    let config = dir.path().join(".mindtape/config.toml");
    let text = std::fs::read_to_string(&config).unwrap();
    let held = format!("{text}\n[go]\neditor = {command:?}\n");
    mindtape::test_support::write_file(dir.path(), ".mindtape/config.toml", &held);

    let stdout = open_first_task_in(&dir, "");

    assert!(stdout.contains(SUSPEND_MARKER), "{stdout}");
    assert!(!dir.path().join("held-ran").exists());
    assert!(stdout.contains("warning:"), "{stdout}");
    assert!(stdout.contains(&command), "{stdout}");
}

/// The risk `truth/tasks/hyperlink-the-handle-column.typ` exists to guard:
/// the `slug#` column's cells now carry an OSC 8 wrap (and, once a row is typed,
/// an invert-colour wrap on top of that), but the key loop reads real key
/// events off the terminal (crossterm), never the rendered screen — so typing
/// a stem hash and Enter must still select and open the right task with
/// hyperlinks active. Mirrors
/// `opening_a_task_suspends_the_tui_and_runs_the_editor`, with
/// `--hyperlinks always` added.
#[test]
fn opening_a_task_still_works_with_hyperlinks_active() {
    let stdout = select_first_task_and_open(" --hyperlinks always");
    assert!(
        stdout.contains("\u{1b}]8;;file://"),
        "the slug# column must actually carry the OSC 8 wrap in this run\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains(SUSPEND_MARKER),
        "typing the stem hash and pressing Enter must still open the editor with \
         hyperlinks active\nstdout:\n{stdout}"
    );
}
