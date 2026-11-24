//! End-to-end tests for `mt allow`: no `[[tasks.waits]]` checker runs until
//! the user records the config's commands, and a changed `run` line asks again.
//! What an un-allowed `[[on.flip]]` `run` does is `flip_hooks_e2e`'s. These
//! tests cover only that `mt allow` records one.
//!
//! Every stub checker is a line that `sh -c` and `cmd /C` both run the same
//! way, which keeps the battery holding on the Windows job too. Each spawn gets
//! its own data home from [`support::isolated_env`], so no test reads or writes
//! the developer's approvals.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

/// The checker every test declares, which leaves a mark and then says the wait
/// is over.
const RUN: &str = "echo run>> runs.txt&& exit 0";

/// A project whose `up` checker runs `run`, holding one task that waits on
/// `up:pr-7`, with `wip` requiring an unblocked task.
fn project(run: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&config(run));
    write_file(
        dir.path(),
        "tasks/drop-the-patch.typ",
        mindtape::typst_task!(
            "(\n  title: \"Drop the patch\",\n  status: proposed(2026, 6, 1),\n  \
         waits-on: (\"up:pr-7\",),\n)\n"
        ),
    );
    dir
}

fn config(run: &str) -> String {
    format!(
        "[tasks]\nroots = [\"tasks\"]\nstatus = [\n  {{ value = \"proposed\" }},\n  \
         {{ value = \"wip\", unblocked = true }},\n  {{ value = \"done\", terminal = true }},\n]\n\n\
         [[tasks.waits]]\nname = \"up\"\nrun = {run:?}\n"
    )
}

fn ran(dir: &Path) -> bool {
    dir.join("runs.txt").exists()
}

/// `mt allow --status` in `dir`, after asserting the set's verdict and the one
/// checker row this battery's config declares.
fn allow_status(dir: &Path, allowed: bool) -> String {
    let status = support::stdout(dir, &["allow", "--status"]);
    let verdict = if allowed { "allowed" } else { "not allowed" };
    assert!(
        status.contains(&format!("{verdict} in this repository and its worktrees:")),
        "{status}"
    );
    assert!(status.contains(&format!("  waits  {RUN}")), "{status}");
    status
}

/// `value`'s site at `index`, compared whole with `expected`.
fn assert_site(value: &serde_json::Value, index: usize, expected: &serde_json::Value) {
    assert_eq!(value["sites"][index], *expected, "{value}");
}

#[test]
fn no_checker_runs_before_mt_allow_and_the_command_warns_once() {
    let dir = project(RUN);

    let (blocked, stderr) = support::listed(dir.path(), "blocked");

    assert!(!ran(dir.path()));
    assert!(
        blocked.contains("waits on: up:pr-7 (unknown: not allowed)"),
        "{blocked}"
    );
    assert_eq!(stderr.matches("warning:").count(), 1, "{stderr}");
    assert!(stderr.contains(&format!("'{RUN}'")), "{stderr}");
    assert!(
        stderr.contains("hint: read them, then run mt allow"),
        "{stderr}"
    );
}

#[test]
fn a_command_that_runs_no_checker_does_not_warn() {
    let dir = project(RUN);
    support::run_ok(dir.path(), &["done", "drop-the-patch", "--force"]);

    let (_, stderr) = support::listed(dir.path(), "");

    assert!(!stderr.contains("warning:"), "{stderr}");
}

#[test]
fn mt_allow_lists_the_commands_and_checkers_run_after_it() {
    let dir = project(RUN);

    let allowed = support::stdout(dir.path(), &["allow"]);
    let (ready, stderr) = support::listed(dir.path(), "ready");

    assert!(
        allowed.contains("allowed in this repository and its worktrees:"),
        "{allowed}"
    );
    assert!(allowed.contains(&format!("  waits  {RUN}")), "{allowed}");
    assert!(ran(dir.path()));
    assert!(ready.contains("Drop the patch"), "{ready}");
    assert!(!stderr.contains("warning:"), "{stderr}");
}

#[test]
fn allow_status_lists_the_commands_without_recording_them() {
    let dir = project(RUN);

    allow_status(dir.path(), false);
    let (blocked, stderr) = support::listed(dir.path(), "blocked");
    assert!(blocked.contains("unknown: not allowed"), "{blocked}");
    assert!(stderr.contains("warning:"), "{stderr}");
    assert!(!ran(dir.path()));
}

#[test]
fn allow_status_json_lists_the_sites_and_the_sets_verdict() {
    let dir = project(RUN);

    let before = support::json(dir.path(), &["allow", "--status", "--json"]);

    support::assert_schema(&before, "allow-status");
    assert_eq!(before["allowed"], false);
    assert_site(
        &before,
        0,
        &serde_json::json!({"kind": "gate", "moment": "waits", "command": RUN}),
    );

    support::allow(dir.path());
    let after = support::json(dir.path(), &["allow", "--status", "--json"]);

    assert_eq!(after["allowed"], true);
    assert_site(
        &after,
        0,
        &serde_json::json!({"kind": "gate", "moment": "waits", "command": RUN}),
    );
}

#[test]
fn allow_status_json_does_not_record() {
    let dir = project(RUN);

    let status = support::json(dir.path(), &["allow", "--json"]);

    support::assert_schema(&status, "allow-status");
    assert_eq!(status["allowed"], false);
    let (blocked, _) = support::listed(dir.path(), "blocked");
    assert!(blocked.contains("unknown: not allowed"), "{blocked}");
    assert!(!ran(dir.path()));
}

#[test]
fn a_config_with_no_commands_prints_the_same_line_in_both_modes() {
    let dir = support::scaffold_project("[tasks]\nroots = [\"tasks\"]\n");

    let status = support::stdout(dir.path(), &["allow", "--status"]);
    let bare = support::stdout(dir.path(), &["allow"]);

    assert_eq!(status, bare);
    assert!(status.contains("nothing to allow"), "{status}");
    let value = support::json(dir.path(), &["allow", "--json"]);

    support::assert_schema(&value, "allow-status");
    assert_eq!(value["allowed"], true);
    assert_eq!(value["sites"], serde_json::json!([]));
}

/// A command written as an array stays an array in JSON, so a script can join
/// it with the config's own `run` value.
#[test]
fn allow_status_json_keeps_an_array_command_as_an_array() {
    let dir = support::scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.waits]]\nname = \"up\"\n\
         run = [\"sh\", \"-c\", \"exit 0\"]\n",
    );

    let value = support::json(dir.path(), &["allow", "--status", "--json"]);

    assert_eq!(
        value["sites"][0]["command"],
        serde_json::json!(["sh", "-c", "exit 0"])
    );
}

/// The `[go]` and `[add]` editors `mt allow` records are part of the status
/// listing too, because one enumerator feeds both modes.
#[test]
fn allow_status_lists_the_editors_bare_allow_records() {
    let dir = support::scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n[go]\neditor = \"zed {task}\"\n",
    );

    let status = support::stdout(dir.path(), &["allow", "--status"]);
    let value = support::json(dir.path(), &["allow", "--status", "--json"]);

    assert!(status.contains("  go  zed {task}"), "{status}");
    assert_site(
        &value,
        0,
        &serde_json::json!({"kind": "editor", "moment": "go", "command": "zed {task}"}),
    );
}

#[test]
fn a_changed_run_line_asks_again() {
    let dir = project(RUN);
    support::allow(dir.path());
    let changed = "echo again>> runs.txt&& exit 0";
    write_file(dir.path(), ".mindtape/config.toml", config(changed));

    let (blocked, stderr) = support::listed(dir.path(), "blocked");

    assert!(!ran(dir.path()));
    assert!(blocked.contains("unknown: not allowed"), "{blocked}");
    assert!(stderr.contains(&format!("'{changed}'")), "{stderr}");
}

#[test]
fn wip_force_still_starts_a_task_whose_checker_is_not_allowed() {
    let dir = project(RUN);

    let stderr = support::refusal(dir.path(), &["wip", "drop-the-patch"]);
    support::run_ok(dir.path(), &["wip", "drop-the-patch", "--force"]);
    assert!(stderr.contains("unknown: not allowed"), "{stderr}");
    assert!(!ran(dir.path()));
    let task = support::task_json(dir.path(), "drop-the-patch");
    assert_eq!(task["status"]["name"], "wip", "{task}");
}

#[test]
fn a_sibling_worktree_shares_the_approval() {
    let dir = project(RUN);
    support::git_commit_all(dir.path());
    let outside = tempdir();
    let sibling = outside.path().join("sibling");
    support::git(
        dir.path(),
        &["worktree", "add", "-q", sibling.to_str().unwrap()],
    );
    let data = outside.path().join("data");
    let in_dir = |at: &Path, args: &[&str]| {
        let out = support::mt(at)
            .env(mindtape::DATA_HOME_ENV, &data)
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
    };

    in_dir(dir.path(), &["allow"]);
    in_dir(&sibling, &["ls", "ready"]);

    assert!(ran(&sibling));
}

#[test]
fn a_config_whose_only_command_is_a_flip_run_has_something_to_allow() {
    let dir = support::scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n[[on.flip]]\nwhen = ''\n\
         actions = [{ run = \"exit 0\", gate = true }]\n",
    );

    let allowed = support::stdout(dir.path(), &["allow"]);

    assert!(!allowed.contains("nothing to allow"), "{allowed}");
    assert!(allowed.contains("  exit 0"), "{allowed}");
}

/// A set member's warning references `mt allow` with the member's `@name` and
/// the member itself. The line then runs as printed from the directory that
/// listed the set.
#[test]
fn a_set_members_warning_asks_for_its_own_mt_allow() {
    let dir = project(RUN);
    let other = support::project_with_tasks(&[("one.typ", "One")]);
    let cfg = tempdir();
    support::write_at_toml(cfg.path(), &[("both", &[dir.path(), other.path()])]);
    let name = dir.path().file_name().unwrap().to_str().unwrap();
    let here = tempdir();
    let run = |args: &[&str]| {
        let mut command = support::mt(here.path());
        command
            .env(mindtape::CONFIG_HOME_ENV, cfg.path())
            .args(args);
        support::parts_of(&command.output().unwrap())
    };

    let (_, _, stderr) = run(&["@both", "ls", "blocked"]);
    let printed = format!("mt @{name} allow");
    let words: Vec<&str> = printed.split(' ').skip(1).collect();
    let (code, allowed, _) = run(&words);

    assert!(
        stderr.contains(&format!(
            "hint: read them, then run {printed}\nnote: command: '{RUN}'\nnote: project: {name}\n"
        )),
        "{stderr}"
    );
    assert_eq!(code, Some(0));
    assert!(allowed.contains(RUN), "{allowed}");
}
