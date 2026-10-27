#![allow(clippy::unwrap_used, clippy::indexing_slicing, unsafe_code)]

use mindtape::test_support::write_file;
use mindtape::test_support::{tempdir, tempdir_with_files};
use std::collections::HashSet;
use std::path::PathBuf;

use mindtape::test_support::bare_task;
use mindtape::{Status, Task};
use swhid_mint::test_support::repo_with;

use super::report::rewrite_link_count;
use super::{Plan, apply, check_eligible_files, inbound_references, plain_delete_plan, plan};
use crate::git;
use typst_cleanup::{Rewrite, is_clean};

/// Build an eligible id set from string slices.
pub(super) fn ids(values: &[&str]) -> HashSet<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

pub(super) fn task(id: &str, file: &str, status: Option<&str>) -> Task {
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
        "#import \"@local/mindtape:0.2.0\": *\n\n\
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
    write_file(&root, "orphan.typ", "orphan\n");
    let tasks = vec![
        task("gone", "gone.typ", Some("done")),
        task("orphan", "orphan.typ", Some("done")),
    ];

    let result = plan(&tasks, &ids(&["gone", "orphan"]), &root, &root);
    assert!(result.is_err());
    assert!(root.join("gone.typ").exists());
    assert!(root.join("orphan.typ").exists());
}

// --- unreferenced removal: no git at all ---

#[test]
fn inbound_references_finds_survivor_edges() {
    // The parse-plane scan sees the `links:` helper edges, not just `link()`.
    let dir = tempdir_with_files(&[("live.typ", &depends_survivor())]);
    let root = dir.path();
    let tasks = gone_and_live("done");

    let referenced = inbound_references(&tasks, &ids(&["gone"]), root).unwrap();

    assert_eq!(referenced, ids(&["gone"]));
}

#[test]
fn inbound_references_empty_when_nothing_points_at_eligible() {
    let dir = tempdir_with_files(&[("live.typ", "no links here\n")]);
    let root = dir.path();
    let tasks = gone_and_live("done");

    let referenced = inbound_references(&tasks, &ids(&["gone"]), root).unwrap();

    assert!(referenced.is_empty());
}

#[test]
fn inbound_references_ignores_eligible_tasks_own_edges() {
    // `a` (also being removed) links to `gone` — both go away in the same run,
    // so the edge is not waiting for attention and must not force a tombstone.
    let dir = tempdir_with_files(&[("a.typ", &depends_survivor())]);
    let root = dir.path();
    let tasks = vec![
        task("gone", "gone.typ", Some("done")),
        task("a", "a.typ", Some("done")),
    ];

    let referenced = inbound_references(&tasks, &ids(&["gone", "a"]), root).unwrap();

    assert!(referenced.is_empty());
}

#[test]
fn plain_delete_plan_deletes_without_git() {
    // No repository anywhere: the eligible task is removed by direct deletion,
    // the survivor is left untouched — not even read.
    let dir = tempdir_with_files(&[("gone.typ", "gone\n"), ("live.typ", &depends_survivor())]);
    let root = dir.path();
    let tasks = gone_and_live("done");

    let plan = plain_delete_plan(&tasks, &ids(&["gone"]), root);
    assert_eq!(plan.deletions.len(), 1);
    assert_eq!(plan.deletions[0].rel, PathBuf::from("gone.typ"));
    assert!(plan.rewrites.is_empty());

    apply(&plan).unwrap();
    assert!(!root.join("gone.typ").exists());
    assert!(root.join("live.typ").exists());
}

#[test]
fn plain_delete_plan_sorts_like_tombstone_path() {
    let dir = tempdir();
    let root = dir.path();
    let tasks = vec![
        task("b", "b.typ", Some("done")),
        task("a", "a.typ", Some("done")),
    ];

    let plan = plain_delete_plan(&tasks, &ids(&["a", "b"]), root);

    assert_eq!(plan.deletions[0].rel, PathBuf::from("a.typ"));
    assert_eq!(plan.deletions[1].rel, PathBuf::from("b.typ"));
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
    assert_eq!(
        mindtape::canonical(&git::toplevel(&sub).unwrap()).unwrap(),
        // `repo_with` resolves its root with std, which keeps `\\?\` on Windows.
        mindtape::canonical(&root).unwrap()
    );
}

#[test]
fn preflight_passes_with_unrelated_dirty_file() {
    // An unrelated file is modified; the eligible task is clean.
    // The pre-flight must not block: only the task files themselves matter.
    let (_dir, root) = repo_with(&[("gone.typ", "gone\n"), ("notes.txt", "draft\n")]);
    // Dirty an unrelated file.
    write_file(&root, "notes.txt", "changed draft\n");
    let tasks = vec![task("gone", "gone.typ", Some("done"))];
    let result = check_eligible_files(&tasks, &ids(&["gone"]), &root, &root);
    assert!(result.is_ok());
}

#[test]
fn preflight_blocks_dirty_eligible_task() {
    let (_dir, root) = repo_with(&[("gone.typ", "gone\n")]);
    write_file(&root, "gone.typ", "changed\n");
    let tasks = vec![task("gone", "gone.typ", Some("done"))];
    let err = check_eligible_files(&tasks, &ids(&["gone"]), &root, &root).unwrap_err();
    crate::diag::assert_plain(
        &err,
        "error: cannot remove: 'gone'\n\
         hint: commit them first: a tombstone pins the last-touch commit\n\
         note: file: gone.typ has uncommitted changes\n",
    );
}

#[test]
fn preflight_blocks_untracked_eligible_task() {
    // Seed with a dummy committed file so git has at least one commit.
    let (_dir, root) = repo_with(&[("seed.txt", "seed\n")]);
    write_file(&root, "orphan.typ", "orphan\n");
    let tasks = vec![task("orphan", "orphan.typ", Some("done"))];
    let err = check_eligible_files(&tasks, &ids(&["orphan"]), &root, &root).unwrap_err();
    let text = crate::diag::plain(&err);
    assert!(
        text.starts_with("error: cannot remove: 'orphan'\n"),
        "{text}"
    );
    assert!(
        text.contains("note: file: orphan.typ is not tracked by git\n"),
        "{text}"
    );
}

#[test]
fn preflight_reports_all_offenders_at_once() {
    // Two tasks with problems: both must appear in the single error, not just the first.
    let (_dir, root) = repo_with(&[("a.typ", "a\n")]);
    write_file(&root, "a.typ", "changed\n");
    write_file(&root, "b.typ", "b\n"); // untracked
    let tasks = vec![
        task("a", "a.typ", Some("done")),
        task("b", "b.typ", Some("done")),
    ];
    let err = check_eligible_files(&tasks, &ids(&["a", "b"]), &root, &root).unwrap_err();
    let text = crate::diag::plain(&err);
    assert!(
        text.starts_with("error: cannot remove several tasks\n"),
        "{text}"
    );
    assert!(
        text.contains("note: file: a.typ has uncommitted changes\n"),
        "{text}"
    );
    assert!(
        text.contains("note: file: b.typ is not tracked by git\n"),
        "{text}"
    );
}

#[test]
fn a_no_answer_aborts_and_a_yes_goes_on() {
    assert!(super::ensure_confirmed(true).is_ok());
    assert_eq!(
        crate::diag::plain(&super::ensure_confirmed(false).unwrap_err()),
        "error: aborted by user\n"
    );
}

#[test]
fn a_task_file_outside_the_repository_is_refused_as_one() {
    let (_dir, root) = repo_with(&[("seed.txt", "seed\n")]);
    let elsewhere = tempdir();
    let tasks = vec![task("gone", "gone.typ", Some("done"))];
    let err = check_eligible_files(&tasks, &ids(&["gone"]), &root, elsewhere.path()).unwrap_err();
    assert!(
        crate::diag::plain(&err).contains("note: file: gone.typ is outside the git repository\n")
    );
}

#[test]
fn rm_without_a_terminal_refuses_and_names_yes() {
    assert!(super::ensure_can_ask(true).is_ok());
    assert_eq!(
        crate::diag::plain(&super::ensure_can_ask(false).unwrap_err()),
        "error: can't ask for confirmation without a terminal\n\
         hint: pass --yes to remove without asking\n"
    );
}

#[test]
fn the_delete_prompt_lists_the_files_and_the_links_it_rewrites() {
    let plan = Plan {
        deletions: vec![typst_cleanup::Deletion {
            abs: PathBuf::from("/r/gone.typ"),
            rel: PathBuf::from("gone.typ"),
        }],
        rewrites: vec![Rewrite {
            rel: PathBuf::from("live.typ"),
            links: 1,
            contents: String::new(),
            abs: PathBuf::from("/r/live.typ"),
        }],
    };
    assert_eq!(
        super::report::prompt_lines(&plan),
        [
            "rm will delete 1 task and rewrite links in 1 file:",
            "  - gone.typ"
        ]
    );
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

#[path = "ask_tests.rs"]
mod ask_tests;

#[path = "cli_tests.rs"]
mod cli_tests;

#[path = "gitless_tests.rs"]
mod gitless_tests;

#[path = "gitless_retire_tests.rs"]
mod gitless_retire_tests;

#[test]
fn no_repository_names_the_way_out_and_redacts_the_root() {
    let dir = tempdir();
    let root = dir.path();
    let git_error = git::toplevel(root).unwrap_err();

    let with_git_said = crate::diag::plain(&super::no_repository(Some(&git_error), root));
    assert!(with_git_said.starts_with(
        "error: cannot rewrite inbound links without a git repository\nhint: set [rm] strategy to \"retire\" or \"unlink\" for gitless repositories\nnote: git said: "
    ), "{with_git_said}");
    assert!(
        !with_git_said.contains(&root.display().to_string()),
        "{with_git_said}"
    );

    let bare = crate::diag::plain(&super::no_repository(None, root));
    assert!(!bare.contains("note:"), "{bare}");
}

/// A tombstone's `path=` is a git path, `/`-separated on every platform. On
/// Windows the cleanup crate hands it over with `\`, which no other machine
/// resolves.
#[test]
fn a_tombstone_writes_its_path_with_forward_slashes() {
    use typst_cleanup::{Artifact, Policy, Swhid};
    let swhid = Swhid {
        kind: swhid_mint::Kind::Revision,
        hash: "0".repeat(40),
        path: Some("tasks\\gone.typ".to_owned()),
    };
    let artifact = Artifact {
        abs: PathBuf::from("gone.typ"),
        rel: PathBuf::from("gone.typ"),
        id: "gone".to_owned(),
    };
    assert_eq!(
        super::TaskLinks.tombstone(&artifact, &swhid),
        format!("\"swh:1:rev:{};path=tasks/gone.typ\"", "0".repeat(40))
    );
}
