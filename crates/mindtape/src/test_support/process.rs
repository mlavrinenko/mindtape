//! Re-running a test in a child process: for state a shared test binary
//! would poison.
#![allow(clippy::disallowed_methods)]

use std::process::{Command, Output};

/// Re-exec this test binary for the single test named `test_name`, with
/// `sentinel` set in the child's environment, after `configure` adds its own
/// env or working directory.
///
/// Process-global state (a `OnceLock`, the `log` global logger) can only be
/// asserted "unset before init" in a fresh process: sharing the test binary's
/// process would race on init order and poison the state for every other test.
/// Callers branch on `std::env::var(sentinel)` — parent side spawns, child side
/// runs the real assertions. `RUST_LOG` is cleared so the child's logging is
/// independent of the developer's environment.
///
/// # Panics
/// Panics if the test binary cannot be spawned, a test-harness bug.
#[allow(clippy::unwrap_used)]
pub fn run_self_test(
    test_name: &str,
    sentinel: &str,
    configure: impl FnOnce(&mut Command),
) -> Output {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .env(sentinel, "1")
        .env_remove("RUST_LOG")
        .args([test_name, "--exact", "--nocapture"]);
    configure(&mut command);
    command.output().unwrap()
}

/// The calling test's full name, which libtest gives the thread it runs on.
///
/// # Panics
/// Panics off a libtest test thread, which has no test name.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn current_test_name() -> String {
    let name = std::thread::current().name().unwrap().to_owned();
    assert_ne!(name, "main", "not on a libtest test thread");
    name
}

/// Run the calling test again, alone, in a fresh process with `sentinel` set
/// ([`run_self_test`]), assert it ran and passed, and return its output. In
/// that child the sentinel is set, so this returns `None` and the test body
/// runs. Callers write `if rerun_isolated(CHILD).is_some() { return; }`.
///
/// # Panics
/// Panics if the child failed or ran no test.
pub fn rerun_isolated(sentinel: &str) -> Option<Output> {
    rerun_isolated_with(sentinel, |_| {})
}

/// [`rerun_isolated`], with `configure` adding the child's env or working
/// directory.
///
/// # Panics
/// Panics if the child failed or ran no test.
pub fn rerun_isolated_with(sentinel: &str, configure: impl FnOnce(&mut Command)) -> Option<Output> {
    if std::env::var(sentinel).is_ok() {
        return None;
    }
    let out = run_self_test(&current_test_name(), sentinel, configure);
    assert_child_ok(&out);
    Some(out)
}

/// Assert a [`run_self_test`] child exited successfully, surfacing its stderr,
/// and that it ran one test: a name that matches none exits 0 too.
///
/// # Panics
/// Panics if either does not hold.
pub fn assert_child_ok(out: &Output) {
    assert!(
        out.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("running 1 test"),
        "the child ran no test: {stdout}"
    );
}
