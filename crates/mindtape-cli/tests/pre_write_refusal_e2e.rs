//! A refused write is reported once: its breaches as `error:` lines, a breach
//! it raises on another task as one `note:` for the whole batch, and no
//! summary of its own.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

mod support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"todo\"\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\n\
     [[tasks.rules]]\nname = \"wip-cap\"\n\
     when = 'status == \"wip\"'\n\
     assert = 'count(status == \"wip\") <= 1'\n\
     severity = \"error\"\n";

fn task(title: &str, status: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  status: {status}(2026, 6, 1),\n)\n"
    )
}

/// One `wip` task, `foo`, filling the cap, and two `todo` ones.
fn project() -> tempfile::TempDir {
    let dir = support::scaffold_project(CONFIG);
    for (stem, status) in [("foo", "wip"), ("bar", "todo"), ("baz", "todo")] {
        std::fs::write(
            dir.path().join(format!("tasks/{stem}.typ")),
            task(stem, status),
        )
        .unwrap();
    }
    dir
}

fn stderr_of(dir: &Path, args: &[&str]) -> String {
    let out = support::mt(dir).args(args).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    String::from_utf8(out.stderr).unwrap()
}

const BREACH: &str = "rules.wip-cap: must satisfy `count(status == \"wip\") <= 1`";

#[test]
fn the_breach_on_another_task_is_one_note_for_the_whole_batch() {
    let dir = project();

    let stderr = stderr_of(dir.path(), &["wip", "bar", "baz"]);

    assert_eq!(
        stderr,
        format!(
            "error: tasks/bar.typ:3: {BREACH}\n\
             note: tasks/foo.typ:3: {BREACH}\n\
             note: task: bar\n\
             error: tasks/baz.typ:3: {BREACH}\n\
             note: task: baz\n\
             error: flipped 0 of 2 tasks\n"
        )
    );
}

#[test]
fn a_single_refused_target_prints_no_summary_and_no_reported_on() {
    let dir = project();

    let stderr = stderr_of(dir.path(), &["wip", "bar"]);

    assert_eq!(
        stderr,
        format!(
            "error: tasks/bar.typ:3: {BREACH}\n\
             note: tasks/foo.typ:3: {BREACH}\n"
        )
    );
}
