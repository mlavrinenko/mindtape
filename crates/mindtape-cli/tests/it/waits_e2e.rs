//! End-to-end tests for `waits-on`: a `[[tasks.waits]]` checker's exit code
//! decides whether a task waits on something outside the repo.
//!
//! Every stub checker is a line that `sh -c` and `cmd /C` both run the same
//! way, so the battery holds on the Windows job too.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use std::path::Path;

use crate::support;

use support::stdout;

/// A project whose `up` checker runs `run`, holding one task that waits on
/// `up:pr-7` and one that waits on nothing.
fn project(run: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&config(run));
    write_task(
        dir.path(),
        "drop-the-patch",
        "Drop the patch",
        r#"("up:pr-7",)"#,
    );
    write_task(dir.path(), "free", "Free", "()");
    dir
}

fn config(run: &str) -> String {
    format!("[tasks]\nroots = [\"tasks\"]\n\n[[tasks.waits]]\nname = \"up\"\nrun = {run:?}\n")
}

fn write_task(dir: &Path, stem: &str, title: &str, waits_on: &str) {
    write_task_in(dir, stem, title, "wip", waits_on);
}

fn write_task_in(dir: &Path, stem: &str, title: &str, status: &str, waits_on: &str) {
    let source = format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  status: {status}(2026, 6, 1),\n  \
         waits-on: {waits_on},\n)\n"
    );
    std::fs::write(dir.join("tasks").join(format!("{stem}.typ")), source).unwrap();
}

fn listed(dir: &Path, filter: &str) -> String {
    stdout(dir, &["ls", filter, "--columns", "title,why"])
}

#[test]
fn a_task_is_blocked_while_its_checker_exits_one_and_why_quotes_its_first_line() {
    let dir = project("echo PR {id} is open&& exit 1");

    let blocked = listed(dir.path(), "blocked");
    let ready = listed(dir.path(), "ready");

    assert!(blocked.contains("Drop the patch"), "{blocked}");
    assert!(
        blocked.contains("waits on: up:pr-7 (PR pr-7 is open)"),
        "{blocked}"
    );
    assert!(!ready.contains("Drop the patch"), "{ready}");
    assert!(ready.contains("Free"), "{ready}");
}

#[test]
fn a_task_is_ready_once_its_checker_exits_zero() {
    let dir = project("exit 0");

    let ready = listed(dir.path(), "ready");

    assert!(ready.contains("Drop the patch"), "{ready}");
    assert!(!listed(dir.path(), "blocked").contains("Drop the patch"));
}

#[test]
fn any_other_exit_code_is_unknown_and_keeps_the_task_blocked() {
    let dir = project("exit 3");

    let blocked = listed(dir.path(), "blocked");

    assert!(
        blocked.contains("waits on: up:pr-7 (unknown: exit 3)"),
        "{blocked}"
    );
}

#[test]
fn a_checker_runs_once_per_distinct_id_however_many_tasks_wait_on_it() {
    let dir = project("echo run>> runs.txt&& exit 1");
    write_task(dir.path(), "also-waits", "Also waits", r#"("up:pr-7",)"#);

    listed(dir.path(), "blocked");

    let runs = std::fs::read_to_string(dir.path().join("runs.txt")).unwrap();
    assert_eq!(runs.lines().count(), 1, "{runs}");
}

#[test]
fn a_finished_task_runs_no_checker() {
    let dir = project("echo run>> runs.txt&& exit 1");
    support::run_ok(dir.path(), &["done", "drop-the-patch", "--force"]);
    std::fs::remove_file(dir.path().join("runs.txt")).ok();

    listed(dir.path(), "");

    assert!(!dir.path().join("runs.txt").exists());
}

#[test]
fn json_why_names_the_entry_and_the_checker_line() {
    let dir = project("echo PR open&& exit 1");

    let task = support::task_json(dir.path(), "drop-the-patch");

    assert_eq!(
        task["why"],
        serde_json::json!([{"stem": "up:pr-7", "status": "PR open", "kind": "waits-on"}])
    );
    assert_eq!(task["waits_on"], serde_json::json!(["up:pr-7"]));
}

#[test]
fn check_refuses_an_entry_naming_no_declared_checker() {
    let dir = project("exit 0");
    write_task(dir.path(), "typo", "Typo", r#"("upp:pr-7",)"#);

    let (code, stderr) = support::run_stderr(dir.path(), &["check"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("unknown checker: 'upp'"), "{stderr}");
}

#[test]
fn check_refuses_an_id_that_could_reach_the_shell() {
    let dir = project("exit 0");
    write_task(dir.path(), "inject", "Inject", r#"("up:x; touch pwned",)"#);

    let (code, stderr) = support::run_stderr(dir.path(), &["check"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("waits-on[0]"), "{stderr}");
    assert!(!dir.path().join("pwned").exists());
}

#[test]
fn set_writes_waits_on_like_tags() {
    let dir = project("exit 1");

    support::run_ok(dir.path(), &["set", "free", "waits-on=up:pr-8"]);

    assert!(listed(dir.path(), "blocked").contains("waits on: up:pr-8"));
}

/// A project whose `wip` requires an unblocked task, holding `drop-the-patch`
/// in `status`, waiting on `up:pr-7` while the checker says the PR is open.
fn guarded_project(status: &str) -> tempfile::TempDir {
    let legend = "status = [\n  { value = \"proposed\" },\n  \
                  { value = \"wip\", unblocked = true },\n  \
                  { value = \"done\", terminal = true },\n]\n";
    let config = config("echo PR open&& exit 1")
        .replace("[[tasks.waits]]", &format!("{legend}\n[[tasks.waits]]"));
    let dir = support::scaffold_project(&config);
    write_task_in(
        dir.path(),
        "drop-the-patch",
        "Drop the patch",
        status,
        r#"("up:pr-7",)"#,
    );
    dir
}

#[test]
fn flip_refuses_a_status_that_requires_an_unblocked_task_while_the_checker_waits() {
    let dir = guarded_project("proposed");

    let (code, stderr) = support::run_stderr(dir.path(), &["wip", "drop-the-patch"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("status requires an unblocked task: 'wip'"),
        "{stderr}"
    );
    assert!(
        stderr.contains("note: waits on: up:pr-7 (PR open)"),
        "{stderr}"
    );
}

#[test]
fn reopening_a_finished_task_asks_its_checker_before_the_guard() {
    let dir = guarded_project("done");

    let (code, stderr) = support::run_stderr(dir.path(), &["wip", "drop-the-patch"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("waits on: up:pr-7 (PR open)"), "{stderr}");
}

/// A committed repository whose `start-only-ready-work` rule refuses a flip
/// into `wip` while the task is blocked, holding `drop-the-patch` as
/// `proposed` and waiting on `up:pr-7`.
fn waiting_repo(run: &str) -> tempfile::TempDir {
    let rule = "\n[[commits.rules]]\nname = \"start-only-ready-work\"\nscope = \"task\"\n\
                when = 'into(\"wip\")'\nassert = 'staged(!blocked)'\n\
                message = \"don't start a task that still waits\"\nseverity = \"error\"\n";
    let dir = support::scaffold_project(&format!("{}{rule}", config(run)));
    write_task_in(
        dir.path(),
        "drop-the-patch",
        "Drop the patch",
        "proposed",
        r#"("up:pr-7",)"#,
    );
    support::git_commit_all(dir.path());
    write_task_in(
        dir.path(),
        "drop-the-patch",
        "Drop the patch",
        "wip",
        r#"("up:pr-7",)"#,
    );
    support::git(dir.path(), &["add", "-A"]);
    support::git(
        dir.path(),
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "start"],
    );
    dir
}

fn check_range(dir: &Path) -> (bool, String) {
    let out = support::mt(dir)
        .args(["check", "--commits", "HEAD~1..HEAD"])
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

#[test]
fn a_commit_range_reads_a_waiting_task_as_blocked() {
    let dir = waiting_repo("echo PR open&& exit 1");

    let (ok, text) = check_range(dir.path());

    assert!(!ok, "{text}");
    assert!(
        text.contains("start-only-ready-work: drop-the-patch"),
        "{text}"
    );
}

#[test]
fn a_commit_range_passes_once_the_checker_says_the_wait_is_over() {
    let dir = waiting_repo("exit 0");

    let (ok, text) = check_range(dir.path());

    assert!(ok, "{text}");
}

#[test]
fn a_commit_range_asks_a_checker_once_however_many_commits_it_holds() {
    let dir = waiting_repo("echo run>> runs.txt&& exit 1");

    check_range(dir.path());

    let runs = std::fs::read_to_string(dir.path().join("runs.txt")).unwrap();
    assert_eq!(runs.lines().count(), 1, "{runs}");
}
