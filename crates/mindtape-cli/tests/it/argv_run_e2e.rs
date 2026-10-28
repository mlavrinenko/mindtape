//! End-to-end tests for the array form of `run`: an `[[on.flip]]` action or a
//! `[[tasks.waits]]` checker written as `["program", "arg", ...]` spawns the
//! program directly, with no shell, the same on every platform.
//!
//! The program each test spawns is `mt` itself, the one binary every runner
//! has, so the battery holds on the Windows job too. Its path goes into the
//! config as a TOML literal string, which keeps a Windows path's backslashes.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use mindtape::test_support::write_file;

use crate::support;

/// The `mt` under test, as a TOML literal string.
fn mt_literal() -> String {
    format!("'{}'", env!("CARGO_BIN_EXE_mt"))
}

/// An array `run` whose program is `mt` and whose arguments are `args`, each a
/// TOML basic string.
fn mt_argv(args: &[&str]) -> String {
    let quoted: Vec<String> = args.iter().map(|arg| format!("{arg:?}")).collect();
    format!("[{}, {}]", mt_literal(), quoted.join(", "))
}

const LEGEND: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\nunblocked = true\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A project holding one `wip` task, `alpha`, that waits on `waits_on`, with
/// `extra` appended to its config and the config's commands allowed.
fn project(extra: &str, waits_on: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&format!("{LEGEND}{extra}"));
    write_file(
        dir.path(),
        "tasks/alpha.typ",
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n  \
             waits-on: {waits_on},\n)\n"
        ),
    );
    support::run_ok(dir.path(), &["allow"]);
    dir
}

/// A gate on the way into `done` that runs `mt` with `args`.
fn gate(args: &[&str]) -> String {
    format!(
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{{ run = {}, gate = true }}]\n",
        mt_argv(args)
    )
}

fn flip_done(dir: &Path) -> (Option<i32>, String) {
    support::run_stderr(dir, &["flip", "done", "alpha"])
}

#[test]
fn an_array_gate_runs_its_program_and_passes_on_exit_zero() {
    let dir = project(&gate(&["--version"]), "()");

    let (code, stderr) = flip_done(dir.path());

    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        std::fs::read_to_string(dir.path().join("tasks/alpha.typ"))
            .unwrap()
            .contains("done("),
    );
}

/// No shell reads an argument: `&` splits a line under both `sh` and `cmd`,
/// so the line `mt bogus & exit 0` would pass, but `mt` gets `bogus & exit 0`
/// as one argument, names it, and exits 2.
#[test]
fn an_array_argument_reaches_the_program_whole() {
    let dir = project(&gate(&["bogus & exit 0"]), "()");

    let out = support::mt(dir.path())
        .args(["flip", "done", "alpha"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(
        stdout.contains("unknown command: 'bogus & exit 0'"),
        "{stdout}"
    );
}

/// A checker's `{id}` fills one array item: `mt help <id>` exits 0 for a
/// command that exists, so the task is ready, and 2 for one that doesn't, so
/// it stays blocked. `mt help` loads no project, so the checker asks no
/// checker of its own.
#[test]
fn an_array_checker_fills_its_id_into_one_argument() {
    let checker = format!(
        "[[tasks.waits]]\nname = \"cmd\"\nrun = {}\n",
        mt_argv(&["help", "{id}"])
    );

    let known = project(&checker, "(\"cmd:dicts\",)");
    let unknown = project(&checker, "(\"cmd:no-such-command\",)");

    let ready = support::stdout(known.path(), &["ls", "ready"]);
    let blocked = support::stdout(unknown.path(), &["ls", "blocked"]);
    assert!(ready.contains("Alpha"), "{ready}");
    assert!(blocked.contains("Alpha"), "{blocked}");
}

/// `mt allow` lists an array in TOML's array syntax, so it reads apart from a
/// shell line.
#[test]
fn mt_allow_lists_an_array_as_an_array() {
    let dir = support::scaffold_project(&format!("{LEGEND}{}", gate(&["--version"])));

    let allowed = support::stdout(dir.path(), &["allow"]);

    let shown = format!(
        "[{}, \"--version\"]",
        serde_json::Value::from(env!("CARGO_BIN_EXE_mt"))
    );
    assert!(allowed.contains(&shown), "{allowed}");
}
