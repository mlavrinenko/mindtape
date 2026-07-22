#![allow(clippy::unwrap_used, clippy::indexing_slicing, unsafe_code)]

use std::collections::HashSet;
use std::path::PathBuf;

use mindtape::test_support::bare_task;
use mindtape::{Config, LoadOutcome, Readiness, Status, Task, parse_config};
use swhid_mint::test_support::repo_with;

use super::{Plan, apply, check_eligible_files, plan, rewrite_link_count, select_targets};
use crate::git;
use crate::util::Project;
use typst_cleanup::{Rewrite, is_clean};

/// Build an eligible id set from string slices.
fn ids(values: &[&str]) -> HashSet<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

fn task(id: &str, file: &str, status: Option<&str>) -> Task {
    let mut t = bare_task(id);
    t.file = PathBuf::from(file);
    t.status = status.map(|name| Status {
        name: name.to_owned(),
        date: None,
        note: None,
    });
    t
}

/// The recurring `gone` (carrying `gone_status`) + `live` task pair, where
/// `live` depends on `gone`.
fn gone_and_live(gone_status: &str) -> Vec<Task> {
    vec![
        task("gone", "gone.typ", Some(gone_status)),
        task("live", "live.typ", None),
    ]
}

/// A survivor whose `links:` field carries `edges` (a raw helper-call
/// expression), wrapped in the task marker shape `mt rm` scans.
fn survivor(edges: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Live\",\n  links: {edges},\n)\n\n== Summary\n\nBody.\n"
    )
}

/// The `depends-on` survivor used across the base cases: one inbound edge to
/// `gone`, authored as a helper call over a plain stem string (never `link(...)`)
/// — the shape the crate's `link()`-only scanner cannot see.
fn depends_survivor() -> String {
    survivor("depends-on(\"gone.typ\")")
}

/// Assert `rewrite` retargeted exactly `links` inbound edge(s) onto `gone`'s
/// SWHID tombstone, and that no un-rewritten reference to `gone.typ` remains.
fn assert_tombstoned(rewrite: &typst_cleanup::Rewrite, links: usize) {
    assert_eq!(rewrite.links, links);
    assert!(rewrite.contents.contains("swh:1:rev:"));
    assert!(rewrite.contents.contains("path=gone.typ"));
    assert!(!rewrite.contents.contains("\"gone.typ\""));
}

#[test]
fn plan_tombstones_removed_and_rewrites_survivor() {
    let (_dir, root) = repo_with(&[
        ("gone.typ", "gone body\n"),
        ("live.typ", &depends_survivor()),
    ]);
    let tasks = gone_and_live("done");

    let plan = plan(&tasks, &ids(&["gone"]), &root, &root).unwrap();

    assert_eq!(plan.deletions.len(), 1);
    assert_eq!(plan.deletions[0].rel, PathBuf::from("gone.typ"));
    assert_eq!(plan.rewrites.len(), 1);
    assert_tombstoned(&plan.rewrites[0], 1);
}

#[test]
fn plan_rewrites_related_and_custom_kind_edges() {
    // A survivor with three inbound edges to `gone` — `depends-on`, the
    // note-bearing `related` escape hatch, and a custom kind via `edge(...)` —
    // has every one retargeted, and the `edge` kind string is left untouched.
    let (_dir, root) = repo_with(&[
        ("gone.typ", "gone\n"),
        (
            "live.typ",
            &survivor(
                "depends-on(\"gone.typ\") + related(\"gone.typ\")[why] + edge(\"blocks\", \"gone.typ\")",
            ),
        ),
    ]);
    let tasks = gone_and_live("done");

    let plan = plan(&tasks, &ids(&["gone"]), &root, &root).unwrap();

    assert_eq!(plan.rewrites.len(), 1);
    let rewrite = &plan.rewrites[0];
    assert_tombstoned(rewrite, 3);
    // The note and the custom kind name survive the retarget verbatim.
    assert!(rewrite.contents.contains("[why]"));
    assert!(rewrite.contents.contains("edge(\"blocks\", "));
}

#[test]
fn apply_writes_and_deletes_without_committing() {
    let (_dir, root) = repo_with(&[("gone.typ", "gone\n"), ("live.typ", &depends_survivor())]);
    let tasks = gone_and_live("done");
    let plan = plan(&tasks, &ids(&["gone"]), &root, &root).unwrap();
    apply(&plan).unwrap();

    assert!(!root.join("gone.typ").exists());
    let live = std::fs::read_to_string(root.join("live.typ")).unwrap();
    assert!(live.contains("swh:1:rev:"));
    // rm must not commit: the tree is left dirty for the user to review.
    assert!(!is_clean(&root).unwrap());
}

#[test]
fn empty_plan_when_nothing_eligible() {
    let (_dir, root) = repo_with(&[("live.typ", &depends_survivor())]);
    let tasks = vec![task("live", "live.typ", Some("wip"))];
    let plan = plan(&tasks, &HashSet::new(), &root, &root).unwrap();
    assert_eq!(plan, Plan::default());
}

#[test]
fn explicit_id_retires_non_terminal_task() {
    // A non-terminal task named in the eligible set is still retired — `rm`
    // bypasses the terminal gate when a target is given.
    let (_dir, root) = repo_with(&[("gone.typ", "gone\n"), ("live.typ", &depends_survivor())]);
    let tasks = gone_and_live("wip");
    let plan = plan(&tasks, &ids(&["gone"]), &root, &root).unwrap();
    assert_eq!(plan.deletions.len(), 1);
    assert_eq!(plan.deletions[0].rel, PathBuf::from("gone.typ"));
}

#[test]
fn plan_aborts_when_a_removed_task_is_untracked() {
    // `gone` is committed; `orphan` exists in the tree but was never committed,
    // so it has no last-touch commit to pin a tombstone to. The whole plan must
    // fail — all-or-nothing, before any deletion.
    let (_dir, root) = repo_with(&[("gone.typ", "gone\n")]);
    std::fs::write(root.join("orphan.typ"), "orphan\n").unwrap();
    let tasks = vec![
        task("gone", "gone.typ", Some("done")),
        task("orphan", "orphan.typ", Some("done")),
    ];

    let result = plan(&tasks, &ids(&["gone", "orphan"]), &root, &root);
    assert!(result.is_err());
    assert!(root.join("gone.typ").exists());
    assert!(root.join("orphan.typ").exists());
}

// The behaviour of `is_clean` (dirty tracked file, untracked file) is owned and
// tested by `typst-cleanup`; the CLI's reliance on it is covered by
// `apply_writes_and_deletes_without_committing` asserting the post-apply tree is
// left dirty.

#[test]
fn toplevel_finds_work_tree_from_subdir() {
    let (_dir, root) = repo_with(&[("a.typ", "a\n")]);
    let sub = root.join("nested");
    std::fs::create_dir_all(&sub).unwrap();
    assert_eq!(git::toplevel(&sub).unwrap().canonicalize().unwrap(), root);
}

#[test]
fn preflight_passes_with_unrelated_dirty_file() {
    // An unrelated file is modified; the eligible task is clean.
    // The pre-flight must not block: only the task files themselves matter.
    let (_dir, root) = repo_with(&[("gone.typ", "gone\n"), ("notes.txt", "draft\n")]);
    // Dirty an unrelated file.
    std::fs::write(root.join("notes.txt"), "changed draft\n").unwrap();
    let tasks = vec![task("gone", "gone.typ", Some("done"))];
    let result = check_eligible_files(&tasks, &ids(&["gone"]), &root, &root);
    assert!(result.is_ok());
}

#[test]
fn preflight_blocks_dirty_eligible_task() {
    let (_dir, root) = repo_with(&[("gone.typ", "gone\n")]);
    std::fs::write(root.join("gone.typ"), "changed\n").unwrap();
    let tasks = vec![task("gone", "gone.typ", Some("done"))];
    let err = check_eligible_files(&tasks, &ids(&["gone"]), &root, &root).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("gone"), "slug missing from error: {msg}");
    assert!(
        msg.contains("uncommitted"),
        "reason missing from error: {msg}"
    );
    // Distinguishes survivor: `==` → `!=`:
    // single-offender format uses `cannot remove {id}:` and singular
    // "the file", not the plural "their files" in the multi-offender path.
    assert!(
        msg.contains("cannot remove gone:"),
        "single-offender format expected, got: {msg}"
    );
    assert!(
        !msg.contains("commit their files first"),
        "single-offender must not use plural path, got: {msg}"
    );
}

#[test]
fn preflight_blocks_untracked_eligible_task() {
    // Seed with a dummy committed file so git has at least one commit.
    let (_dir, root) = repo_with(&[("seed.txt", "seed\n")]);
    std::fs::write(root.join("orphan.typ"), "orphan\n").unwrap();
    let tasks = vec![task("orphan", "orphan.typ", Some("done"))];
    let err = check_eligible_files(&tasks, &ids(&["orphan"]), &root, &root).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("orphan"), "slug missing from error: {msg}");
    assert!(
        msg.contains("not tracked"),
        "reason missing from error: {msg}"
    );
}

#[test]
fn preflight_reports_all_offenders_at_once() {
    // Two tasks with problems: both must appear in the single error, not just the first.
    let (_dir, root) = repo_with(&[("a.typ", "a\n")]);
    std::fs::write(root.join("a.typ"), "changed\n").unwrap();
    std::fs::write(root.join("b.typ"), "b\n").unwrap(); // untracked
    let tasks = vec![
        task("a", "a.typ", Some("done")),
        task("b", "b.typ", Some("done")),
    ];
    let err = check_eligible_files(&tasks, &ids(&["a", "b"]), &root, &root).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains('a'), "task a missing: {msg}");
    assert!(msg.contains('b'), "task b missing: {msg}");
}

#[test]
fn rewrite_link_count_sums_all_rewrites() {
    // Catches survivors: `-> usize` replaced with `0` or `1`.
    let plan = Plan {
        rewrites: vec![
            Rewrite {
                rel: PathBuf::from("a.typ"),
                links: 3,
                contents: String::new(),
                abs: PathBuf::from("/a.typ"),
            },
            Rewrite {
                rel: PathBuf::from("b.typ"),
                links: 5,
                contents: String::new(),
                abs: PathBuf::from("/b.typ"),
            },
        ],
        deletions: vec![],
    };
    assert_eq!(rewrite_link_count(&plan), 8);
}

/// An in-memory project — no disk needed for slug/filter selection logic.
fn stub_project(config: Config, tasks: Vec<Task>) -> Project {
    Project {
        root: PathBuf::from("."),
        config,
        outcome: LoadOutcome {
            tasks,
            errors: vec![],
        },
    }
}

fn readiness_of(project: &Project) -> Readiness {
    Readiness::compute(&project.outcome.tasks, &project.config)
}

#[test]
fn select_targets_ref_bypasses_status() {
    // A slug reference selects exactly that task, even though it is non-terminal.
    let project = stub_project(
        Config::default(),
        vec![
            task("gone", "tasks/gone.typ", Some("wip")),
            task("live", "tasks/live.typ", Some("wip")),
        ],
    );
    let got = select_targets(&project, Some("gone"), &readiness_of(&project)).unwrap();
    assert_eq!(got, ids(&["gone"]));
}

#[test]
fn select_targets_filter_selects_every_match() {
    let project = stub_project(
        Config::default(),
        vec![
            task("a", "tasks/a.typ", Some("done")),
            task("b", "tasks/b.typ", Some("wip")),
            task("c", "tasks/c.typ", Some("done")),
        ],
    );
    let got = select_targets(
        &project,
        Some("status == \"done\""),
        &readiness_of(&project),
    )
    .unwrap();
    assert_eq!(got, ids(&["a", "c"]));
}

#[test]
fn select_targets_default_filter_is_terminal() {
    // No target -> the built-in `terminal` default filter sweeps terminal tasks.
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap();
    let project = stub_project(
        config,
        vec![
            task("a", "tasks/a.typ", Some("done")),
            task("b", "tasks/b.typ", Some("wip")),
        ],
    );
    let got = select_targets(&project, None, &readiness_of(&project)).unwrap();
    assert_eq!(got, ids(&["a"]));
}

#[test]
fn select_targets_unresolvable_token_errors() {
    // Not a slug and not a valid boolean filter -> error, not a silent empty set.
    let project = stub_project(
        Config::default(),
        vec![task("a", "tasks/a.typ", Some("wip"))],
    );
    assert!(select_targets(&project, Some("nope"), &readiness_of(&project)).is_err());
}

#[path = "rm_cli_tests.rs"]
mod rm_cli_tests;
