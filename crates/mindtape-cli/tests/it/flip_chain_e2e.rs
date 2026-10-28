//! End-to-end tests for a `[[on.flip]]` command that runs `mt flip` itself.
//!
//! A hook may flip another task, and that task's own hooks run as usual. A
//! hook that flips the task and status it runs for would start the same hook
//! again, without end, so `mt` refuses the nested flip and names the loop.
//! Every flip here runs under a timeout, so a regression fails the suite
//! instead of hanging it. The hooks run the `mt` under test as an array, so
//! the battery holds on the Windows job too.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;
use std::time::Duration;

use mindtape::test_support::write_file;

use crate::support;

const LEGEND: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A gate on `stem`'s way into `done` that runs `mt flip done <target>`.
fn closes(stem: &str, target: &str) -> String {
    format!(
        "[[on.flip]]\nwhen = 'id == \"{stem}\" && into(\"done\")'\n\
         actions = [{{ run = ['{}', 'flip', 'done', '{target}'], gate = true }}]\n",
        env!("CARGO_BIN_EXE_mt")
    )
}

/// A project of `wip` tasks named `stems`, with `hooks` in its config and the
/// config's commands allowed.
fn project(stems: &[&str], hooks: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&format!("{LEGEND}{hooks}"));
    for stem in stems {
        write_file(
            dir.path(),
            format!("tasks/{stem}.typ"),
            mindtape::typst_task!("(\n  title: \"A task\",\n  status: wip(2026, 6, 1),\n)\n"),
        );
    }
    support::run_ok(dir.path(), &["allow"]);
    dir
}

/// `mt flip done <stem>` under a timeout: the exit code and stdout then stderr.
fn flip_done(dir: &Path, stem: &str) -> (Option<i32>, String) {
    let out = support::mt(dir)
        .args(["flip", "done", stem])
        .timeout(Duration::from_secs(20))
        .output()
        .unwrap();
    let (_, text) = support::text_of(&out);
    (out.status.code(), text)
}

fn is_done(dir: &Path, stem: &str) -> bool {
    std::fs::read_to_string(dir.join(format!("tasks/{stem}.typ")))
        .unwrap()
        .contains("done(")
}

/// The nested `mt flip done alpha` is refused in one hop, which fails the
/// outer gate, so the flip ends with a refusal and writes nothing.
#[test]
fn a_hook_that_flips_its_own_task_is_refused_in_one_hop() {
    let dir = project(&["alpha"], &closes("alpha", "alpha"));

    let (code, text) = flip_done(dir.path(), "alpha");

    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("flip already in progress"), "{text}");
    assert!(text.contains("alpha → done"), "{text}");
    assert!(!is_done(dir.path(), "alpha"), "{text}");
}

/// `alpha`'s gate closes `beta`, and `beta`'s own gate closes `gamma`: a
/// chain of three distinct flips, each running its task's gate.
#[test]
fn a_hook_that_flips_another_task_runs_that_tasks_gate() {
    let hooks = closes("alpha", "beta") + &closes("beta", "gamma");
    let dir = project(&["alpha", "beta", "gamma"], &hooks);

    let (code, text) = flip_done(dir.path(), "alpha");

    assert_eq!(code, Some(0), "{text}");
    for stem in ["alpha", "beta", "gamma"] {
        assert!(is_done(dir.path(), stem), "{stem}: {text}");
    }
}

/// Two hooks that close each other loop through two tasks; the second visit
/// to `alpha` is refused.
#[test]
fn hooks_that_flip_each_other_are_refused() {
    let hooks = closes("alpha", "beta") + &closes("beta", "alpha");
    let dir = project(&["alpha", "beta"], &hooks);

    let (code, text) = flip_done(dir.path(), "alpha");

    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("flip already in progress"), "{text}");
    assert!(!is_done(dir.path(), "alpha"), "{text}");
    assert!(!is_done(dir.path(), "beta"), "{text}");
}
