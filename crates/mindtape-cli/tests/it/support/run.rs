//! The `mt` spawns and read-backs every e2e module shares: a run's exit code,
//! stdout and stderr, the assertions on a refusal, and the fixtures a run
//! reads back. Re-exported by [`super`]. A test reaches them as
//! `support::run` and friends.
#![allow(clippy::unwrap_used, clippy::disallowed_methods, dead_code)]

use std::path::Path;

use super::mt;

/// `mt <args>` in `dir`: the finished run, for a test that reads its exit
/// code, stdout and stderr itself.
pub fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    mt(dir).args(args).output().unwrap()
}

/// `mt <args>` in `dir`: the exit code and stderr.
pub fn run_stderr(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let output = run(dir, args);
    (
        output.status.code(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

/// What a finished `mt` (or `git`) run printed, stdout then stderr, with
/// whether it exited zero: the pair most failure assertions read.
pub fn text_of(out: &std::process::Output) -> (bool, String) {
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

/// A finished run's exit code, stdout and stderr, each read as UTF-8.
pub fn parts_of(out: &std::process::Output) -> (Option<i32>, String, String) {
    (
        out.status.code(),
        String::from_utf8(out.stdout.clone()).unwrap(),
        String::from_utf8(out.stderr.clone()).unwrap(),
    )
}

/// `mt <args>` in `dir`: the exit code, stdout and stderr, each read as UTF-8.
pub fn parts(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    parts_of(&run(dir, args))
}

/// `mt <args>` in `dir`, asserting the refusal exit code 1: stdout and stderr,
/// for the caller to match on.
pub fn refusal_parts(dir: &Path, args: &[&str]) -> (String, String) {
    let (code, stdout, stderr) = parts(dir, args);
    assert_eq!(code, Some(1), "{stderr}");
    (stdout, stderr)
}

/// `mt <args>` in `dir`, as `(succeeded, stdout then stderr)`.
pub fn run_text(dir: &Path, args: &[&str]) -> (bool, String) {
    text_of(&run(dir, args))
}

/// `mt check` in `dir` must fail and print `needle`.
pub fn assert_check_fails(dir: &Path, needle: &str) {
    let (ok, text) = run_text(dir, &["check"]);
    assert!(!ok, "check passed, wanted a failure: {text}");
    assert!(
        text.contains(needle),
        "check output lacks {needle:?}: {text}"
    );
}

/// `mt <args>` in `dir` must exit with `code` and print exactly `expected` on
/// stderr: the shape of every error-format test.
pub fn assert_stderr(dir: &Path, args: &[&str], code: i32, expected: &str) {
    let (got, stderr) = run_stderr(dir, args);
    assert_eq!(got, Some(code), "{stderr}");
    assert_eq!(stderr, expected);
}

/// `mt <args>` in `dir`, asserting the refusal exit code 1: the stderr, for
/// the caller to match on.
pub fn refusal(dir: &Path, args: &[&str]) -> String {
    let (code, stderr) = run_stderr(dir, args);
    assert_eq!(code, Some(1), "{stderr}");
    stderr
}

/// `mt <args>` in `dir`, asserting it failed: the stderr, for the caller to
/// match on. `why` names what the refusal proves.
pub fn failed(dir: &Path, args: &[&str], why: &str) -> String {
    let out = run(dir, args);
    assert!(!out.status.success(), "{why}: {out:?}");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Run `mt` in `dir` and assert it succeeds.
pub fn run_ok(dir: &Path, args: &[&str]) {
    mt(dir).args(args).assert().success();
}

/// [`run_ok`] for a test that reads what the run printed: the finished run.
pub fn ok(dir: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    mt(dir).args(args).assert().success()
}

/// Run `mt` in `dir` and assert it failed: the shape of a refusal test that
/// only needs the exit status.
pub fn assert_fails(dir: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    mt(dir).args(args).assert().failure()
}

/// `mt <args>` in `dir` with `stdin` on its standard input, asserting it
/// succeeded: the finished run, for a test that reads what it printed.
pub fn ok_stdin(dir: &Path, args: &[&str], stdin: &str) -> assert_cmd::assert::Assert {
    mt(dir).args(args).write_stdin(stdin).assert().success()
}

/// Assert `mt check` passes in `dir`: what a command wrote still validates.
pub fn check_ok(dir: &Path) {
    run_ok(dir, &["check"]);
}

/// Run `mt add --name <name> <args…>` in `dir`, assert it succeeds, and
/// return the task file it wrote, `tasks/<name>.typ`.
pub fn add_and_read(dir: &Path, name: &str, args: &[&str]) -> String {
    let mut command = vec!["add", "--name", name];
    command.extend_from_slice(args);
    run_ok(dir, &command);
    std::fs::read_to_string(dir.join("tasks").join(format!("{name}.typ"))).unwrap()
}

/// Run `mt` in `dir`, assert it succeeds, and parse its stdout as JSON.
pub fn json(dir: &Path, args: &[&str]) -> serde_json::Value {
    serde_json::from_str(&stdout(dir, args)).unwrap()
}

/// Run `mt` in `dir`, assert it succeeds, and return its stdout. This is
/// [`run_ok`] for the tests that assert on what the run printed rather than only
/// that it succeeded.
pub fn stdout(dir: &Path, args: &[&str]) -> String {
    let out = run(dir, args);
    assert!(out.status.success(), "mt {args:?} failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Run `mt add --name wired --set title=T --json` in `dir`, returning stdout.
pub fn add_wired_json(dir: &Path) -> String {
    stdout(
        dir,
        &["add", "--name", "wired", "--set", "title=T", "--json"],
    )
}

/// File a task in `dir` via `mt add --name <name> --set title=T <extra_set…>`,
/// then assert `mt ls` and `mt check` both succeed against it. Shared by the
/// `init` e2e batteries that prove a scaffolded preset or answer set is usable
/// end to end, not just that `init` wrote *something*.
pub fn add_then_ls_and_check(dir: &Path, name: &str, extra_set: &[&str]) {
    let mut args = vec!["add", "--name", name, "--set", "title=T"];
    args.extend_from_slice(extra_set);
    run_ok(dir, &args);
    run_ok(dir, &["ls"]);
    check_ok(dir);
}

/// [`add_then_ls_and_check`], then flip the task to `done`: the prologue of
/// every test that proves a finished task still lists and validates.
pub fn add_then_done(dir: &Path, name: &str) {
    add_then_ls_and_check(dir, name, &[]);
    run_ok(dir, &["flip", "done", name]);
}

/// Run `mt allow` in `dir`: approve the gates the project declares.
pub fn allow(dir: &Path) {
    run_ok(dir, &["allow"]);
}

/// Run `mt add <args>` in `dir`, asserting it succeeded.
pub fn add(dir: &Path, args: &[&str]) {
    let mut full = vec!["add"];
    full.extend_from_slice(args);
    run_ok(dir, &full);
}

/// Flip `name` to `status` in `dir`, asserting it succeeded.
pub fn flip(dir: &Path, status: &str, name: &str) {
    run_ok(dir, &["flip", status, name]);
}

/// Run `mt <args>` in `dir` with `MINDTAPE_NOW` pinned to `now`, asserting it
/// succeeded: the shape of every dated-write test.
pub fn run_at(dir: &Path, now: &str, args: &[&str]) {
    mt(dir)
        .env("MINDTAPE_NOW", now)
        .args(args)
        .assert()
        .success();
}

/// Flip `name` to `done` in `dir`, through the `mt done` shortcut.
pub fn flip_done(dir: &Path, name: &str) {
    run_ok(dir, &["done", name]);
}

/// `mt ls <filter> --columns title,why` in `dir`: stdout and stderr.
pub fn listed(dir: &Path, filter: &str) -> (String, String) {
    let out = run(dir, &["ls", filter, "--columns", "title,why"]);
    assert!(out.status.success(), "{out:?}");
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

/// Assert `dir` holds `rel`, the file a write leaves behind.
pub fn assert_exists(dir: &Path, rel: &str) {
    assert!(dir.join(rel).exists(), "{rel} must exist");
}

/// [`assert_exists`] with `why` naming what the file proves.
pub fn assert_exists_because(dir: &Path, rel: &str, why: &str) {
    assert!(dir.join(rel).exists(), "{rel} must exist: {why}");
}

/// `text` with each of `paths` in it in the separator `mt` shows on this
/// platform. The result is an expected message that names them.
pub fn shown(text: &str, paths: &[&str]) -> String {
    super::native_in(text, paths)
}

/// The stderr of a finished `assert_cmd` run, for the tests that match on a
/// refusal's text.
pub fn stderr_of(done: &assert_cmd::assert::Assert) -> String {
    String::from_utf8_lossy(&done.get_output().stderr).into_owned()
}

/// The stdout of a finished `assert_cmd` run, for the tests that match on
/// what a command printed.
pub fn stdout_of(done: &assert_cmd::assert::Assert) -> String {
    String::from_utf8_lossy(&done.get_output().stdout).into_owned()
}
