//! End-to-end proof that an estimate budget and cap are plain `[[tasks.rules]]`
//! entries (`truth/tasks/retire-the-drift-check-and-estimate-bounds-in-favour-of-rules.typ`).
//!
//! Drives a real git working copy so `changed()` reads a real change set. No
//! estimate check runs unless a rule asks for it: a project declaring an
//! estimate unit and nothing else sees no warning however far the working tree
//! outgrows the task.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use mindtape::test_support::git_init;

mod support;

/// A project config declaring a `src-files` estimate unit, with `rules`
/// appended verbatim.
fn config(rules: &str) -> String {
    format!(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\n\
         [tasks.estimate]\nunits = [\"src-files\"]\n\n{rules}"
    )
}

/// The working-tree budget, counting everything but `gen/**`.
const BUDGET: &str = "[[tasks.rules]]\nname = \"src-budget\"\n\
     when = 'status == \"wip\"'\n\
     assert = 'changed(\"staged+worktree\", \"!gen/**\") <= estimate.src-files'\n\
     message = \"split the remaining work into a new task\"\n";

/// The ceiling on a declared estimate, a validation error.
const CAP: &str = "[[tasks.rules]]\nname = \"cap\"\n\
     assert = 'estimate.src-files <= 5'\nseverity = \"error\"\n";

/// One `wip` task budgeting `files` source files, written as raw harvest
/// metadata so the fixture needs no `@local/mindtape` package install.
fn task_src(files: u32) -> String {
    format!(
        "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Budgeted task\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 7, day: 22), note: none),
    estimate: (src-files: {files}),
  ),
  extras: (:),
))
"
    )
}

/// A git working copy holding a project under `rules` whose single in-flight
/// task budgets `files` files, plus `extra` untracked files on top of the two
/// the scaffold itself writes and one under the excluded `gen/`.
fn fixture(rules: &str, files: u32, extra: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_init(root);
    for sub in [".mindtape", "tasks", "gen"] {
        std::fs::create_dir_all(root.join(sub)).unwrap();
    }
    std::fs::write(root.join(".mindtape/config.toml"), config(rules)).unwrap();
    std::fs::write(root.join("tasks/budgeted.typ"), task_src(files)).unwrap();
    std::fs::write(root.join("gen/OUTPUT.md"), "generated\n").unwrap();
    for n in 0..extra {
        std::fs::write(root.join(format!("scratch-{n}.txt")), "x\n").unwrap();
    }
    dir
}

/// Run `mt check` in `root`, returning its exit success and stdout.
fn check(root: &Path) -> (bool, String) {
    let output = support::mt(root).arg("check").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (output.status.success(), stdout)
}

#[test]
fn an_outgrown_working_tree_warns_through_the_budget_rule() {
    // 2 scaffold files plus 3 scratch files count; `gen/OUTPUT.md` does not.
    let dir = fixture(BUDGET, 4, 3);
    let (ok, stdout) = check(dir.path());
    assert!(ok, "a warn rule leaves `mt check` green: {stdout}");
    assert!(
        stdout
            .contains("warning: tasks/budgeted.typ:1: rules.src-budget: split the remaining work"),
        "the breach names the rule and its advice: {stdout}"
    );
}

#[test]
fn an_excluded_path_does_not_spend_the_budget() {
    // 5 counted paths against a 5-file budget; the excluded sixth is free.
    let dir = fixture(BUDGET, 5, 3);
    let (ok, stdout) = check(dir.path());
    assert!(ok && !stdout.contains("src-budget"), "{stdout}");
}

#[test]
fn no_estimate_check_runs_without_a_rule() {
    let dir = fixture("", 1, 8);
    let (ok, stdout) = check(dir.path());
    assert!(ok && stdout.contains("ok: 1 task(s) valid"), "{stdout}");
    assert!(!stdout.contains("working tree"), "{stdout}");
}

#[test]
fn an_oversized_estimate_fails_the_cap_rule() {
    let dir = fixture(CAP, 9, 0);
    let (ok, stdout) = check(dir.path());
    assert!(!ok, "an error rule fails the check: {stdout}");
    assert!(
        stdout.contains("rules.cap: must satisfy `estimate.src-files <= 5`"),
        "{stdout}"
    );
}

#[test]
fn the_retired_estimate_tables_are_unknown_keys() {
    for table in [
        "[tasks.estimate.drift]\nunit = \"src-files\"\n",
        "[[tasks.estimate.max]]\nsrc-files = 5\n",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".mindtape")).unwrap();
        std::fs::create_dir_all(root.join("tasks")).unwrap();
        std::fs::write(root.join(".mindtape/config.toml"), config(table)).unwrap();
        let (ok, stdout) = check(root);
        assert!(!ok, "`{table}` must be refused: {stdout}");
    }
}
