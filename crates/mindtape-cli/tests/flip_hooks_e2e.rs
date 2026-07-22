//! End-to-end tests for `[[on.flip]]` hooks, driven through the `mt` binary.
//!
//! The load-bearing one is [`read_commands_execute_nothing`]: `mt flip` is the
//! only command allowed to run a configured command, and a read that shells out
//! would be a surprise in every editor and commit hook `mt check` runs inside.
//! Every other test here pins a gate's effect on the write.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

const TASK: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
     #show: task.with(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n)\n\n\
     == Summary\n\nBody.\n";

const LEGEND: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A project holding one `wip` task and `hooks` appended to its config.
fn project(hooks: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&format!("{LEGEND}{hooks}"));
    std::fs::write(dir.path().join("tasks/alpha.typ"), TASK).unwrap();
    dir
}

fn cmd(dir: &Path, args: &[&str]) -> assert_cmd::Command {
    let mut command = assert_cmd::Command::cargo_bin("mt").unwrap();
    command.current_dir(dir).args(args);
    command
}

fn task_source(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("tasks/alpha.typ")).unwrap()
}

/// `alpha`'s status note read back through the harvest — the only way to tell a
/// note that survived the write from markup Typst quietly interpreted away.
/// Listed with an explicit match-all filter, since the default view hides a
/// task that has reached a terminal status.
fn status_note(dir: &Path) -> String {
    let output = cmd(dir, &["ls", "", "--json"])
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

/// A hook that leaves `fired` behind when its command runs.
const SENTINEL_HOOK: &str = "[[on.flip]]\nwhen = ''\n\
                             actions = [{ run = \"touch fired\", gate = true }]\n";

/// The safety property of the whole feature: only `mt flip` runs actions.
///
/// The hook fires on every transition, so a single read command reaching the
/// runner would leave the sentinel behind. The trailing `mt flip` proves the
/// sentinel mechanism works at all — without it this test would pass on a
/// hook that never ran for anyone.
#[test]
fn read_commands_execute_nothing() {
    let dir = project(SENTINEL_HOOK);
    let sentinel = dir.path().join("fired");
    for args in [
        &["check"][..],
        &["ls"],
        &["ls", "--json"],
        &["ls", "--format", "csv"],
        &["views"],
        &["graph"],
        &["config", "show"],
        &["eval", "tasks/alpha.typ"],
    ] {
        cmd(dir.path(), args).assert().success();
        assert!(
            !sentinel.exists(),
            "`mt {}` executed a configured command",
            args.join(" ")
        );
    }

    cmd(dir.path(), &["flip", "done", "alpha"])
        .assert()
        .success();
    assert!(sentinel.exists(), "flip must run the action");
}

#[test]
fn a_passing_gate_allows_the_flip() {
    let dir = project(
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{ run = \"true\", gate = true }]\n",
    );
    cmd(dir.path(), &["flip", "done", "alpha"])
        .assert()
        .success();
    assert!(task_source(dir.path()).contains("done("));
}

#[test]
fn a_failing_gate_refuses_the_flip_and_writes_nothing() {
    let dir = project(
        "[[on.flip]]\nwhen = 'into(\"done\")'\n\
         actions = [{ run = 'echo needs-review >&2; exit 2', gate = true, \
         message = \"a human has to look first\" }]\n",
    );
    let before = task_source(dir.path());
    let assertion = cmd(dir.path(), &["flip", "done", "alpha"])
        .assert()
        .failure();
    let output = assertion.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("needs-review"),
        "output must surface: {stdout}"
    );
    assert!(
        stderr.contains("a human has to look first"),
        "the refusal must carry the message: {stderr}"
    );
    assert_eq!(before, task_source(dir.path()), "nothing may be written");
}

/// `--force` waives the gate, and the waiver lands in the status note so it is
/// still visible afterwards.
#[test]
fn force_waives_the_gate_and_records_it_in_the_note() {
    let dir = project(
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{ run = \"false\", gate = true }]\n",
    );
    cmd(dir.path(), &["flip", "done", "alpha", "--force"])
        .assert()
        .success();
    let source = task_source(dir.path());
    assert!(source.contains("forced past failing gate"), "{source}");
    assert!(
        source.contains("false"),
        "the waived command is named: {source}"
    );

    // The note is real content, not just text in the file: it reads back.
    let note = status_note(dir.path());
    assert!(note.contains("forced past failing gate"), "{note}");
}

/// A `-m` note keeps its text; the waiver is appended to it.
#[test]
fn a_waiver_is_appended_to_an_authored_note() {
    let dir = project(
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{ run = \"false\", gate = true }]\n",
    );
    cmd(
        dir.path(),
        &["flip", "done", "alpha", "--force", "-m", "shipped by hand"],
    )
    .assert()
    .success();
    let note = status_note(dir.path());
    assert!(note.starts_with("shipped by hand"), "{note}");
    assert!(note.contains("forced past failing gate"), "{note}");
}

/// `into(...)` scopes a hook to one target status.
#[test]
fn into_scopes_the_hook_to_its_target_status() {
    let dir = project(
        "[[tasks.status]]\nvalue = \"blocked\"\n\
         [[on.flip]]\nwhen = 'into(\"done\")'\n\
         actions = [{ run = \"touch fired\", gate = true }]\n",
    );
    cmd(dir.path(), &["flip", "blocked", "alpha"])
        .assert()
        .success();
    assert!(!dir.path().join("fired").exists(), "wrong target fired");

    cmd(dir.path(), &["flip", "done", "alpha"])
        .assert()
        .success();
    assert!(dir.path().join("fired").exists());
}

/// Actions run in declaration order, and a failed gate skips the rest.
#[test]
fn actions_run_in_order_and_stop_at_a_failed_gate() {
    let dir = project(
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [\
         { run = \"touch first\" }, \
         { run = \"false\", gate = true }, \
         { run = \"touch third\" }]\n",
    );
    cmd(dir.path(), &["flip", "done", "alpha"])
        .assert()
        .failure();
    assert!(
        dir.path().join("first").exists(),
        "the first action must run"
    );
    assert!(
        !dir.path().join("third").exists(),
        "a failed gate must skip what follows it"
    );
}

/// Piped stdout suppresses `show` and `list` — but the gate still runs, which
/// is the whole point of separating presentation from enforcement.
#[test]
fn show_and_list_are_silent_when_piped_but_gates_still_run() {
    let dir = project(
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [\
         { show = \"SENTINEL-TEXT\" }, \
         { list = '', title = \"SENTINEL-TITLE\" }, \
         { run = \"touch fired\", gate = true }]\n",
    );
    let assertion = cmd(dir.path(), &["flip", "done", "alpha"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assertion.get_output().stdout).into_owned();
    assert!(!stdout.contains("SENTINEL-TEXT"), "{stdout}");
    assert!(!stdout.contains("SENTINEL-TITLE"), "{stdout}");
    assert!(dir.path().join("fired").exists(), "the gate must still run");
}

/// A malformed hook is a config error at load, so it fails every command
/// equally rather than surprising the first flip that reaches it.
#[test]
fn a_malformed_hook_fails_at_load() {
    let dir = project("[[on.flip]]\nwhen = 'into(\"shipped\")'\nactions = [{ show = \"x\" }]\n");
    cmd(dir.path(), &["check"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("names no configured status"));
}
