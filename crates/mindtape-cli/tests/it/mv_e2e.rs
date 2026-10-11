//! End-to-end battery for `mt mv` — renaming a task's file stem, which is its
//! identity, and retargeting every inbound `links:` edge onto the new one.
//!
//! Lives here (not an inline `#[path]` unit test) so it exercises the real
//! prelude edge helpers (`depends-on`, `related`, `edge`), a real git repo (the
//! move goes through `git mv` when there is one), and the compiled `mt` binary
//! end-to-end — then proves `mt check` stays green, which is the whole point: a
//! rename that missed an inbound edge leaves a dangling one.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

/// A tasks root plus the custom `blocks` edge kind the referrer's
/// `edge("blocks", …)` needs to resolve.
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n";

/// The task being renamed.
const SUBJECT: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Subject\",\n  \
status: wip(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n";

/// The stem `mv` must refuse to land on.
const TAKEN: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Taken\",\n  \
status: wip(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n";

/// Three inbound edges to `old`, one per shape the retarget must reach: a plain
/// `depends-on`, the note-bearing `related`, and a custom kind through `edge`.
const REFERRER: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Referrer\",\n  \
status: wip(2026, 6, 1),\n  \
links: depends-on(\"old.typ\") + related(\"old.typ\")[why] + edge(\"blocks\", \"old.typ\"),\n\
)\n\n== Summary\n\nBody.\n";

/// An inbound edge from a sub-folder, authored as a relative path. Only the
/// last component names the task, so the `../` prefix must survive the rewrite.
const NESTED: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Nested\",\n  \
status: wip(2026, 6, 1),\n  \
links: depends-on(\"../old.typ\"),\n\
)\n\n== Summary\n\nBody.\n";

fn mt(dir: &Path) -> assert_cmd::Command {
    support::mt(dir)
}

/// The project every arm starts from: the subject, its three referrers (one of
/// them in a sub-folder), and a task already holding the `taken` stem.
fn scaffold(dir: &Path) {
    support::scaffold_config(
        dir,
        CONFIG,
        &[
            ("old.typ", SUBJECT),
            ("taken.typ", TAKEN),
            ("referrer.typ", REFERRER),
        ],
    );
    std::fs::create_dir_all(dir.join("tasks/sub")).unwrap();
    std::fs::write(dir.join("tasks/sub/nested.typ"), NESTED).unwrap();
}

fn read(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap()
}

#[test]
fn mv_renames_the_file_and_retargets_every_inbound_edge() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    support::git_commit_all(dir.path());

    mt(dir.path())
        .args(["mv", "old", "new"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tasks/new.typ"))
        .stdout(predicates::str::contains("4 link(s) in 2 file(s)"));

    assert!(!dir.path().join("tasks/old.typ").exists());
    assert!(dir.path().join("tasks/new.typ").exists());

    let referrer = read(dir.path(), "tasks/referrer.typ");
    assert!(
        !referrer.contains("\"old.typ\""),
        "an edge still names the old stem:\n{referrer}"
    );
    assert_eq!(
        referrer.matches("\"new.typ\"").count(),
        3,
        "every edge must be retargeted:\n{referrer}"
    );
    // The splice touches the target literal and nothing else.
    assert!(referrer.contains("[why]"), "note dropped:\n{referrer}");
    assert!(
        referrer.contains("edge(\"blocks\", "),
        "kind dropped:\n{referrer}"
    );

    // Only the last path component names the task; the `../` prefix stays.
    let nested = read(dir.path(), "tasks/sub/nested.typ");
    assert!(
        nested.contains("\"../new.typ\""),
        "relative prefix lost:\n{nested}"
    );

    // The dangling-edge regression: no inbound edge points at a missing task.
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn mv_stages_the_rename_through_git_when_the_tree_is_a_repository() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    support::git_commit_all(dir.path());

    mt(dir.path()).args(["mv", "old", "new"]).assert().success();

    let staged = std::process::Command::new("git")
        .current_dir(dir.path())
        .args(["diff", "--cached", "--name-status", "-M"])
        .output()
        .unwrap();
    let staged = String::from_utf8_lossy(&staged.stdout).into_owned();
    assert!(
        staged.contains("tasks/new.typ") && staged.contains("tasks/old.typ"),
        "the rename was not staged:\n{staged}"
    );
}

#[test]
fn mv_falls_back_to_a_plain_rename_without_a_repository() {
    // No `git init`: the move is an ordinary filesystem rename and the inbound
    // rewrite is unchanged — the same split `mt rm` makes between its git and
    // gitless paths.
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    mt(dir.path()).args(["mv", "old", "new"]).assert().success();

    assert!(dir.path().join("tasks/new.typ").exists());
    assert!(!dir.path().join("tasks/old.typ").exists());
    assert!(read(dir.path(), "tasks/referrer.typ").contains("\"new.typ\""));
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn mv_refuses_a_taken_stem_before_moving_anything() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    mt(dir.path())
        .args(["mv", "old", "taken"])
        .assert()
        .failure()
        .stderr(predicates::str::diff(
            "error: stem is taken: 'taken'\n\
             hint: pick another stem\n\
             note: held by: tasks/taken.typ\n",
        ));

    assert!(dir.path().join("tasks/old.typ").exists());
    assert!(read(dir.path(), "tasks/referrer.typ").contains("\"old.typ\""));
}

#[test]
fn mv_dry_run_prints_the_plan_and_leaves_the_tree_untouched() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    let before = read(dir.path(), "tasks/referrer.typ");

    mt(dir.path())
        .args(["mv", "old", "new", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tasks/referrer.typ"))
        .stdout(predicates::str::contains("dry run: nothing written"));

    assert!(dir.path().join("tasks/old.typ").exists());
    assert!(!dir.path().join("tasks/new.typ").exists());
    assert_eq!(read(dir.path(), "tasks/referrer.typ"), before);
}

#[test]
fn mv_refuses_a_stem_carrying_a_path_separator() {
    // Moving a task between folders changes its derived tags and is a decision
    // of its own; `mv` renames in place and says so rather than guessing.
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    mt(dir.path())
        .args(["mv", "old", "sub/new"])
        .assert()
        .failure();

    assert!(dir.path().join("tasks/old.typ").exists());
}
