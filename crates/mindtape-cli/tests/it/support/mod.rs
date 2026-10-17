//! Shared scaffolding for CLI end-to-end tests (`tests/it/*_e2e.rs`).
//!
//! A module of the one `it` test binary, which declares it in `main.rs`; the
//! e2e modules reach it with `use crate::support;`. A helper that no module
//! calls is dead code, hence the module-wide allow. The project fixtures live in
//! `fixtures.rs`, the merge-driver runs in `merge_driver.rs`, and the scrubbed
//! process builder in `shell.rs`; the `mt` spawns and read-backs stay here.
//! Only Unix e2e modules use `merge_driver.rs` and `shell.rs`, so both build on
//! Unix alone.
#![allow(clippy::unwrap_used, clippy::disallowed_methods, dead_code)]

use std::path::{Path, PathBuf};

mod fixtures;
#[cfg(unix)]
mod merge_driver;
#[cfg(unix)]
mod shell;
pub use fixtures::{
    commit_repo, git, git_commit_all, init_preset, install_lib, project_with_raw_slot_template,
    project_with_slot_template, project_with_task, project_with_tasks, project_with_template,
    project_with_template_and_config, scaffold, scaffold_config, scaffold_project,
    write_aliased_task,
};
#[cfg(unix)]
pub use merge_driver::{merge_branches, run_driver, text_merge_conflicts};
#[cfg(unix)]
pub use shell::{checkout, shell};

const PRELUDE: &str = include_str!("../../../../../lib/prelude.typ");

/// Where [`isolated_env`] points the global-config lookup: a path under the
/// test's own project directory that nothing ever creates.
const NO_GLOBAL_CONFIG: &str = "no-global-config";

/// Where [`isolated_env`] points the data directory: a path under the test's
/// own project directory, which `mt allow` creates when a test runs it.
const DATA_HOME: &str = "data-home";

/// Environment variables that steer `mt` and must not reach a spawned binary
/// from the developer's shell: `MINDTAPE_NOW` repins the clock every dated
/// write reads, `MINDTAPE_WORKTREE` retargets the project root, and `VISUAL`
/// would hand an accidental editor the developer's real one. `DIRENV_DIFF` and
/// `DIRENV_WATCHES` are direnv's record of what it loaded: inherited, they make
/// a `direnv exec` inside a scratch repo rewind `PATH` to the shell's pre-load
/// value, so a script resolves the host's `mt` instead of the one under test.
const CLEARED_VARS: [&str; 5] = [
    "MINDTAPE_NOW",
    "MINDTAPE_WORKTREE",
    "VISUAL",
    "DIRENV_DIFF",
    "DIRENV_WATCHES",
];

/// The editor every isolated spawn resolves instead of the developer's: a
/// command that cannot exist, so an `add` a test did not mean to make
/// interactive fails on the spawn rather than opening a real editor. Tests
/// that exercise the editor set their own `EDITOR` after [`isolate`].
const STUB_EDITOR: &str = "mt-no-such-editor";

/// The environment a spawned `mt` needs for a test to be about the code rather
/// than about the machine running it.
///
/// `mt` layers a global `defaults.toml` under every project config —
/// `dirs::config_dir()/mindtape/defaults.toml`, a supported and documented
/// layer (`crates/mindtape/src/config/layers.rs`). A spawned binary reads
/// whatever the developer happens to keep there, so without this every e2e
/// test is a function of their personal `mt` setup: two lines of `[ls]
/// hyperlinks` in `~/.config/mindtape/defaults.toml` are enough to fail
/// `hyperlinks_e2e` on a tree with no Rust change in it.
///
/// So point that lookup at an empty path. `MINDTAPE_CONFIG_HOME` is the one
/// knob that does it on every platform (`mindtape::config_home`): `dirs` reads
/// the environment only on Unix — `XDG_CONFIG_HOME` on Linux, `HOME` on macOS
/// (which reads `~/Library/Application Support` and ignores XDG) — and goes
/// through a known-folder API on Windows that no variable overrides. The XDG
/// pair stays set anyway, since `HOME` also caps the ancestor
/// `.mindtape/defaults.toml` walk at `dir`, so a stray one above the tempdir
/// cannot leak in either.
///
/// `MINDTAPE_DATA_HOME` gets a fresh directory under `dir` for the same reason:
/// it holds the `mt allow` approvals, and a test must neither read the
/// developer's nor leave its own behind.
pub fn isolated_env(dir: &Path) -> [(&'static str, PathBuf); 4] {
    [
        (mindtape::CONFIG_HOME_ENV, dir.join(NO_GLOBAL_CONFIG)),
        (mindtape::DATA_HOME_ENV, dir.join(DATA_HOME)),
        ("XDG_CONFIG_HOME", dir.join(NO_GLOBAL_CONFIG)),
        ("HOME", dir.to_path_buf()),
    ]
}

/// `mt <args>` in `dir`: the exit code and stderr.
pub fn run_stderr(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let output = mt(dir).args(args).output().unwrap();
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

/// Write `<cfg>/mindtape/at.toml` with one `[at.<name>]` entry per
/// `(name, projects)`, each including the projects' directories.
pub fn write_at_toml(cfg: &Path, entries: &[(&str, &[&Path])]) {
    std::fs::create_dir_all(cfg.join("mindtape")).unwrap();
    let toml: String = entries
        .iter()
        .map(|(name, dirs)| {
            let include: Vec<String> = dirs
                .iter()
                .map(|dir| format!("\"{}\"", dir.display().to_string().replace('\\', "/")))
                .collect();
            format!("[at.{name}]\ninclude = [{}]\n\n", include.join(", "))
        })
        .collect();
    std::fs::write(cfg.join("mindtape/at.toml"), toml).unwrap();
}

/// `mt <args>` in `dir`, as `(succeeded, stdout then stderr)`.
pub fn run_text(dir: &Path, args: &[&str]) -> (bool, String) {
    text_of(&mt(dir).args(args).output().unwrap())
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

/// `mt`, ready to run in `dir` with the developer's global config out of reach
/// ([`isolated_env`]). The spawn every e2e test goes through; a bare
/// `cargo_bin("mt")` is the un-isolated one and only `install_e2e` (which
/// injects its own XDG directories) has cause to write it.
pub fn mt(dir: &Path) -> assert_cmd::Command {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_mt"));
    command.current_dir(dir);
    isolate(&mut command, dir);
    assert_cmd::Command::from_std(command)
}

/// [`mt`]'s isolation for a caller that builds its own command — the pty tests
/// spawn `mt` through `script(1)`, which passes its environment down.
///
/// The editor pair is scrubbed here, not defaulted: a sweep mutant that
/// wrongly answers "open the editor" (`cmd::add::interactive::should_open -> true`
/// and friends) otherwise spawns whatever `$VISUAL`/`$EDITOR` the developer's
/// session carries. That happened: `cargo-mutants` ran the add e2e tests with
/// a `--wait` GUI editor in `VISUAL`, one window per test until the 60 s timeout. In a
/// tree whose suite must be about the code, the developer's editor can never
/// be the fallback; [`STUB_EDITOR`] makes the mistake a fast spawn error.
pub fn isolate<'a>(
    command: &'a mut std::process::Command,
    dir: &Path,
) -> &'a mut std::process::Command {
    for (key, value) in isolated_env(dir) {
        command.env(key, value);
    }
    for key in CLEARED_VARS {
        command.env_remove(key);
    }
    command.env("EDITOR", STUB_EDITOR);
    command
}

/// Every character that opens Typst syntax, in one line of otherwise ordinary
/// prose: emphasis (`*`, `_`), math (`$`), code (`#`), content-block delimiters
/// (`[`, `]`), a label (`<a>`) and a reference (`@at`). Text carrying these must
/// survive a write unchanged — unescaped, `create-*star*` fails to load and
/// `<a>`/`@at` silently evaluate away.
pub const MARKUP_HAZARDS: &str = "create-*star* _u_ $x$ #h [b] <a> >g @at";

/// The canonical-JSON row `mt ls --json` reports for task `id` in `dir`. Reading
/// a value back through the harvest is the only way to tell text that survived a
/// write from text Typst quietly interpreted on the way in.
pub fn task_json(dir: &Path, id: &str) -> serde_json::Value {
    let output = mt(dir)
        .args(["ls", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    parsed
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .unwrap()
        .iter()
        .find(|task| task.get("id").and_then(serde_json::Value::as_str) == Some(id))
        .unwrap_or_else(|| panic!("no task {id:?} in {parsed}"))
        .clone()
}

/// The lone task's status note read back through the harvest, via an
/// explicit match-all filter (`mt ls "" --json`) since the default view hides
/// a task that has already reached a terminal status. Reading it back is the
/// only way to tell a note that survived the write from markup Typst quietly
/// interpreted away.
pub fn status_note(dir: &Path) -> String {
    let output = mt(dir)
        .args(["ls", "", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    parsed
        .pointer("/tasks/0/status/note")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("no status note in {parsed}"))
        .to_owned()
}

/// The value at RFC-6901 `pointer` inside a [`task_json`] row (`/links/0/note`).
/// Spelled as a lookup rather than `row["links"][0]`, which the workspace's
/// `indexing_slicing` lint denies.
pub fn at<'a>(row: &'a serde_json::Value, pointer: &str) -> &'a serde_json::Value {
    row.pointer(pointer)
        .unwrap_or_else(|| panic!("no {pointer} in {row}"))
}

/// Run `mt` in `dir` and assert it succeeds. Named `run_ok` (not `mt`) so it
/// sits alongside [`mt`] rather than shadowing it: callers that need the raw
/// `assert_cmd::Command` (a failure assertion, stdout capture, …) still reach
/// for [`mt`] directly.
pub fn run_ok(dir: &Path, args: &[&str]) {
    mt(dir).args(args).assert().success();
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

/// Run `mt` in `dir`, assert it succeeds, and return its stdout — [`run_ok`]
/// for the tests that assert on what was printed rather than only that it was.
pub fn stdout(dir: &Path, args: &[&str]) -> String {
    let out = mt(dir).args(args).output().unwrap();
    assert!(out.status.success(), "mt {args:?} failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
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

/// Parsed `add --describe=json` schema rows run in `dir`.
pub fn describe_slots(dir: &Path) -> serde_json::Value {
    let output = mt(dir)
        .args(["add", "--describe=json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

/// Slot names from `add --describe=json` run in `dir`.
pub fn describe_slot_names(dir: &Path) -> Vec<String> {
    describe_slots(dir)
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

/// The `kind` reported for slot `name` in `add --describe=json` run in `dir`.
pub fn describe_slot_kind(dir: &Path, name: &str) -> String {
    describe_slots(dir)
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row.get("name").and_then(serde_json::Value::as_str) == Some(name))
        .and_then(|row| row.get("kind"))
        .and_then(serde_json::Value::as_str)
        .unwrap()
        .to_owned()
}

/// `mt check --commit-msg` over `message`, as a hook would run it: its
/// combined output, and whether it exited zero.
pub fn check_commit_msg(dir: &Path, message: &str) -> (bool, String) {
    let file = dir.join("COMMIT_MSG");
    std::fs::write(&file, message).unwrap();
    run_text(dir, &["check", "--commit-msg", file.to_str().unwrap()])
}

/// [`check_commit_msg`] for a message the check must refuse: asserts the
/// refusal and returns the output for the caller to match on.
pub fn commit_msg_refusal(dir: &Path, message: &str) -> String {
    let (ok, text) = check_commit_msg(dir, message);
    assert!(!ok, "the check must refuse {message:?}: {text}");
    text
}

/// The stderr of a finished `assert_cmd` run, for the tests that match on a
/// refusal's text.
pub fn stderr_of(done: &assert_cmd::assert::Assert) -> String {
    String::from_utf8_lossy(&done.get_output().stderr).into_owned()
}
