//! End-to-end proof that the working-tree drift check warns and only warns.
//!
//! Drives a real git working copy so the check reads a real change set, then
//! asserts `mt check` prints the breach *and still exits zero* — the whole point
//! of `truth/tasks/files-budget-drift-check.typ` is that an outgrown task is a
//! rescope, not a failure.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::test_support::git_init;

mod support;

/// A project config declaring a `files` estimate unit, with the drift settings
/// left at their defaults (unit `files`, in-flight `status == "wip"`).
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\n\
     [tasks.estimate]\nunits = [\"files\"]\n";

/// One `wip` task budgeting two files, written as raw harvest metadata so the
/// fixture needs no `@local/mindtape` package install.
fn task_src(files: u32) -> String {
    format!(
        "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Budgeted task\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 7, day: 22), note: none),
    estimate: (files: {files}),
  ),
  extras: (:),
))
"
    )
}

/// A git working copy holding a project whose single in-flight task budgets
/// `files` files, plus `extra` untracked files on top of the two the scaffold
/// itself writes.
fn fixture(files: u32, extra: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_init(root);
    std::fs::create_dir_all(root.join(".mindtape")).unwrap();
    std::fs::create_dir_all(root.join("tasks")).unwrap();
    std::fs::write(root.join(".mindtape/config.toml"), CONFIG).unwrap();
    std::fs::write(root.join("tasks/budgeted.typ"), task_src(files)).unwrap();
    for n in 0..extra {
        std::fs::write(root.join(format!("scratch-{n}.txt")), "x\n").unwrap();
    }
    dir
}

/// Run `mt check` in `root` and return its stdout, asserting a zero exit.
fn check_stdout(root: &Path) -> String {
    let output = support::mt(root).arg("check").output().unwrap();
    assert!(
        output.status.success(),
        "`mt check` must stay green: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn an_outgrown_working_tree_warns_and_still_exits_zero() {
    // 2 scaffold files (config + task) plus 8 scratch files, against a 3-file
    // budget: git sees 10 untracked paths.
    let dir = fixture(3, 8);
    let stdout = check_stdout(dir.path());
    assert!(
        stdout.contains("warning: working tree touches 10 file(s), over the 3"),
        "the breach must be reported: {stdout}"
    );
    assert!(
        stdout.contains("declared by `budgeted`"),
        "one task in flight must be named: {stdout}"
    );
    assert!(
        stdout.contains("split the remaining work into a new task"),
        "the message must name the next move: {stdout}"
    );
}

#[test]
fn a_working_tree_inside_the_budget_says_nothing() {
    let dir = fixture(9, 0);
    let stdout = check_stdout(dir.path());
    assert!(
        !stdout.contains("working tree"),
        "a change set inside the budget is silent: {stdout}"
    );
}

#[test]
fn a_generated_output_does_not_inflate_the_reported_count() {
    // `truth/tasks/exclude-generated-outputs-from-the-estimate-drift-check.typ`:
    // a docs edit that also writes a generated artifact must be counted
    // against its own source only. 2 scaffold files (config + task) plus one
    // real edit plus one excluded generated output, against a 2-file budget:
    // git sees 4 untracked paths, but only 3 count.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_init(root);
    let config = format!("{CONFIG}\n[tasks.estimate.drift]\nexclude = [\"docs/generated/**\"]\n");
    std::fs::create_dir_all(root.join(".mindtape")).unwrap();
    std::fs::create_dir_all(root.join("tasks")).unwrap();
    std::fs::create_dir_all(root.join("docs/generated")).unwrap();
    std::fs::write(root.join(".mindtape/config.toml"), config).unwrap();
    std::fs::write(root.join("tasks/budgeted.typ"), task_src(2)).unwrap();
    std::fs::write(root.join("docs/generated/OUTPUT.md"), "generated\n").unwrap();
    std::fs::write(root.join("real.rs"), "fn main() {}\n").unwrap();

    let stdout = check_stdout(root);
    assert!(
        stdout.contains("working tree touches 3 file(s), over the 2"),
        "the generated doc must not inflate the count — 4 untracked paths, \
         3 counted, over the 2-file budget: {stdout}"
    );
}

#[test]
fn a_project_outside_git_says_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".mindtape")).unwrap();
    std::fs::create_dir_all(root.join("tasks")).unwrap();
    std::fs::write(root.join(".mindtape/config.toml"), CONFIG).unwrap();
    std::fs::write(root.join("tasks/budgeted.typ"), task_src(1)).unwrap();

    // No `git init` here, so the check has no change set to read. It must
    // degrade silently — not warn, and not fail.
    let stdout = check_stdout(root);
    assert!(stdout.contains("ok: 1 task(s) valid"), "{stdout}");
}
