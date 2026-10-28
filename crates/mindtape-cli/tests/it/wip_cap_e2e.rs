//! End-to-end proof that a work-in-progress cap is a plain `[[tasks.rules]]`
//! entry over `count(...)`
//! (`truth/tasks/retire-task-limits-in-favour-of-count-rules.typ`), the shape
//! `[[tasks.limits]]` used to be its own table and checker for.
//!
//! A breach still refuses `mt flip` at `severity = "error"`, exactly as the
//! limit did, and still names every task in the capped set: the candidate
//! being flipped reports its own breach, and the already-`wip` task elsewhere
//! in the set comes back as a note on the refusal
//! ([`crate::validate::candidate::elsewhere_violations`], generalized from the link-cycle
//! case to cover any `[[tasks.rules]]` breach a batch opens or closes on a task
//! it never touches).
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

const LEGEND: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"todo\"\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\n";

/// The WIP cap this repo's own `.mindtape/config.toml` declares, at one.
const CAP: &str = "[[tasks.rules]]\nname = \"wip-cap\"\n\
     when = 'status == \"wip\"'\n\
     assert = 'count(status == \"wip\") <= 1'\n\
     severity = \"error\"\n";

fn task(title: &str, status: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  status: {status}(2026, 6, 1),\n)\n"
    )
}

/// A project under `CAP` with one `wip` task (`foo`) and one `todo` task
/// (`bar`).
fn project() -> tempfile::TempDir {
    let dir = support::scaffold_project(&format!("{LEGEND}{CAP}"));
    write_file(dir.path(), "tasks/foo.typ", task("Foo", "wip"));
    write_file(dir.path(), "tasks/bar.typ", task("Bar", "todo"));
    dir
}

fn flip(dir: &Path, status: &str, stem: &str) -> std::process::Output {
    support::mt(dir)
        .args(["flip", status, stem])
        .output()
        .unwrap()
}

#[test]
fn a_second_wip_task_is_refused_and_names_the_one_already_in_flight() {
    let dir = project();
    let output = flip(dir.path(), "wip", "bar");
    assert!(!output.status.success(), "a cap breach fails the flip");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr,
        shown(
            "error: tasks/bar.typ:3: rules.wip-cap: must satisfy `count(status == \"wip\") <= 1`\n\
         note: tasks/foo.typ:3: rules.wip-cap: must satisfy `count(status == \"wip\") <= 1`\n"
        ),
        "the candidate's own breach is the error; the task already occupying the cap is one note"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/bar.typ")).unwrap(),
        task("Bar", "todo"),
        "nothing was written"
    );
}

#[test]
fn the_first_wip_task_is_unaffected() {
    let dir = project();
    let output = flip(dir.path(), "done", "foo");
    assert!(
        output.status.success(),
        "leaving the only wip task never breaches the cap: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn the_retired_limits_table_is_an_unknown_key() {
    let dir = support::scaffold_project(&format!(
        "{LEGEND}[[tasks.limits]]\nfilter = 'status == \"wip\"'\nmax = 1\n"
    ));
    write_file(dir.path(), "tasks/foo.typ", task("Foo", "wip"));
    let output = support::mt(dir.path()).arg("check").output().unwrap();
    assert!(!output.status.success(), "`tasks.limits` must be refused");
}

/// An expected message, each task path in it in the separator `mt` shows.
fn shown(text: &str) -> String {
    support::native_in(text, &["tasks/bar.typ", "tasks/foo.typ"])
}
