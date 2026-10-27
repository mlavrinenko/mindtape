//! End-to-end tests for `[[on.flip]]` hooks, driven through the `mt` binary.
//!
//! The load-bearing one is [`read_commands_run_no_flip_action`]: `mt flip` is
//! the only command that runs an action, and a read that ran one would be a
//! surprise in every editor and commit hook `mt check` runs inside. A read may
//! run a `[[tasks.waits]]` checker, which `waits_e2e` covers.
//! Every other test here pins a gate's effect on the write, and the last
//! battery what a `run` does before `mt allow` records the config's commands.
//! Its stub commands run the same under `sh -c` and `cmd /C`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

const TASK: &str = mindtape::typst_task!(
    "(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n)\n\n\
     == Summary\n\nBody.\n"
);

const LEGEND: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A project holding one `wip` task and `hooks` appended to its config, with
/// the config's commands allowed.
fn project(hooks: &str) -> tempfile::TempDir {
    let dir = unallowed(hooks);
    support::run_ok(dir.path(), &["allow"]);
    dir
}

/// [`project`] before anyone ran `mt allow`.
fn unallowed(hooks: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&format!("{LEGEND}{hooks}"));
    write_file(dir.path(), "tasks/alpha.typ", TASK);
    dir
}

fn cmd(dir: &Path, args: &[&str]) -> assert_cmd::Command {
    let mut command = support::mt(dir);
    command.args(args);
    command
}

fn task_source(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("tasks/alpha.typ")).unwrap()
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
fn read_commands_run_no_flip_action() {
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
         actions = [{ run = 'echo needs-review >&2 && exit 2', gate = true, \
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
    assert_eq!(
        stderr,
        "error: gate failed: 'echo needs-review >&2 && exit 2': exit 2\n\
         hint: a human has to look first\n\
         hint: or pass --force to skip the gate; the status note records it\n",
        "one error, the config's message, the way past"
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
    let note = support::status_note(dir.path());
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
    let note = support::status_note(dir.path());
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
    let dir = unallowed("[[on.flip]]\nwhen = 'into(\"shipped\")'\nactions = [{ show = \"x\" }]\n");
    cmd(dir.path(), &["check"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown status: 'shipped'"));
}

/// A command that leaves a mark, the same under `sh -c` and `cmd /C`.
const MARK: &str = "echo run>> runs.txt";

fn ran(dir: &Path) -> bool {
    dir.join("runs.txt").exists()
}

fn gate_hook(gate: bool) -> String {
    format!(
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{{ run = {MARK:?}, gate = {gate} }}]\n"
    )
}

#[test]
fn an_unallowed_gate_refuses_the_flip_without_running_it() {
    let dir = unallowed(&gate_hook(true));
    let before = task_source(dir.path());

    let (code, stderr) = support::run_stderr(dir.path(), &["flip", "done", "alpha"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(!ran(dir.path()), "an unallowed gate must not run");
    assert!(
        stderr.contains(&format!("error: gate not allowed: '{MARK}'")),
        "{stderr}"
    );
    assert!(stderr.contains("then run mt allow"), "{stderr}");
    assert!(stderr.contains("or pass --force"), "{stderr}");
    assert_eq!(stderr.matches("warning:").count(), 1, "{stderr}");
    assert_eq!(before, task_source(dir.path()), "nothing may be written");
}

#[test]
fn force_waives_an_unallowed_gate_without_running_it() {
    let dir = unallowed(&gate_hook(true));

    support::run_ok(dir.path(), &["flip", "done", "alpha", "--force"]);

    assert!(!ran(dir.path()), "--force must not run an unallowed gate");
    let note = support::status_note(dir.path());
    assert!(note.contains("forced past failing gate"), "{note}");
    assert!(note.contains(MARK), "{note}");
}

#[test]
fn an_unallowed_run_without_gate_is_skipped_and_says_so() {
    let dir = unallowed(&gate_hook(false));

    let assertion = cmd(dir.path(), &["flip", "done", "alpha"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assertion.get_output().stdout).into_owned();

    assert!(!ran(dir.path()), "an unallowed run must not run");
    assert!(
        stdout.contains(&format!("skipped: {MARK} (not allowed)")),
        "{stdout}"
    );
    assert!(task_source(dir.path()).contains("done("));
}

#[test]
fn mt_allow_lists_a_flip_command_and_it_runs_after() {
    let dir = unallowed(&gate_hook(true));

    let allowed = support::stdout(dir.path(), &["allow"]);
    let (code, stderr) = support::run_stderr(dir.path(), &["flip", "done", "alpha"]);

    assert!(allowed.contains(MARK), "{allowed}");
    assert_eq!(code, Some(0), "{stderr}");
    assert!(!stderr.contains("warning:"), "{stderr}");
    assert!(ran(dir.path()), "an allowed gate runs");
}
