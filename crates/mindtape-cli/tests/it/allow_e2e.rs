//! End-to-end tests for `mt allow`: no `[[tasks.waits]]` checker runs until
//! the user records the config's commands, and a changed `run` line asks again.
//! What an un-allowed `[[on.flip]]` `run` does is `flip_hooks_e2e`'s; here only
//! that `mt allow` records one.
//!
//! Every stub checker is a line that `sh -c` and `cmd /C` both run the same
//! way, so the battery holds on the Windows job too. Each spawn gets its own
//! data home from [`support::isolated_env`], so no test reads or writes the
//! developer's approvals.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use mindtape::test_support::tempdir;
use std::path::Path;

use crate::support;

/// The checker every test declares: it leaves a mark, then says the wait is
/// over.
const RUN: &str = "echo run>> runs.txt&& exit 0";

/// A project whose `up` checker runs `run`, holding one task that waits on
/// `up:pr-7`, with `wip` requiring an unblocked task.
fn project(run: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&config(run));
    std::fs::write(
        dir.path().join("tasks/drop-the-patch.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Drop the patch\",\n  status: proposed(2026, 6, 1),\n  \
         waits-on: (\"up:pr-7\",),\n)\n",
    )
    .unwrap();
    dir
}

fn config(run: &str) -> String {
    format!(
        "[tasks]\nroots = [\"tasks\"]\nstatus = [\n  {{ value = \"proposed\" }},\n  \
         {{ value = \"wip\", unblocked = true }},\n  {{ value = \"done\", terminal = true }},\n]\n\n\
         [[tasks.waits]]\nname = \"up\"\nrun = {run:?}\n"
    )
}

/// `mt ls <filter> --columns title,why` in `dir`: stdout and stderr.
fn listed(dir: &Path, filter: &str) -> (String, String) {
    let out = support::mt(dir)
        .args(["ls", filter, "--columns", "title,why"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn ran(dir: &Path) -> bool {
    dir.join("runs.txt").exists()
}

#[test]
fn no_checker_runs_before_mt_allow_and_the_command_warns_once() {
    let dir = project(RUN);

    let (blocked, stderr) = listed(dir.path(), "blocked");

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

    let (_, stderr) = listed(dir.path(), "");

    assert!(!stderr.contains("warning:"), "{stderr}");
}

#[test]
fn mt_allow_lists_the_commands_and_checkers_run_after_it() {
    let dir = project(RUN);

    let allowed = support::stdout(dir.path(), &["allow"]);
    let (ready, stderr) = listed(dir.path(), "ready");

    assert!(allowed.contains(RUN), "{allowed}");
    assert!(ran(dir.path()));
    assert!(ready.contains("Drop the patch"), "{ready}");
    assert!(!stderr.contains("warning:"), "{stderr}");
}

#[test]
fn a_changed_run_line_asks_again() {
    let dir = project(RUN);
    support::run_ok(dir.path(), &["allow"]);
    let changed = "echo again>> runs.txt&& exit 0";
    std::fs::write(dir.path().join(".mindtape/config.toml"), config(changed)).unwrap();

    let (blocked, stderr) = listed(dir.path(), "blocked");

    assert!(!ran(dir.path()));
    assert!(blocked.contains("unknown: not allowed"), "{blocked}");
    assert!(stderr.contains(&format!("'{changed}'")), "{stderr}");
}

#[test]
fn wip_force_still_starts_a_task_whose_checker_is_not_allowed() {
    let dir = project(RUN);

    let (code, stderr) = support::run_stderr(dir.path(), &["wip", "drop-the-patch"]);
    support::run_ok(dir.path(), &["wip", "drop-the-patch", "--force"]);

    assert_eq!(code, Some(1), "{stderr}");
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
