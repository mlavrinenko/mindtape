#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use mindtape::{Status, Task};

use super::{Plan, apply, plan};
use crate::git;

/// Run `git -C dir args`, asserting success.
fn git_run(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

/// A temp repo with `files` written, all committed in one commit.
fn repo_with(files: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    git_run(&root, &["init", "-q"]);
    git_run(&root, &["config", "user.email", "t@example.com"]);
    git_run(&root, &["config", "user.name", "Test"]);
    git_run(&root, &["config", "commit.gpgsign", "false"]);
    for (rel, contents) in files {
        let abs = root.join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&abs, contents).unwrap();
    }
    git_run(&root, &["add", "-A"]);
    git_run(&root, &["commit", "-q", "-m", "seed"]);
    (dir, root)
}

fn task(id: &str, file: &str, status: Option<&str>) -> Task {
    Task {
        id: id.to_owned(),
        file: PathBuf::from(file),
        title: id.to_owned(),
        status: status.map(|name| Status {
            name: name.to_owned(),
            date: None,
            note: None,
        }),
        priority: None,
        grade: None,
        kind: None,
        estimate_hours: None,
        depends_on: Vec::new(),
        relates_to: Vec::new(),
        tags: Vec::new(),
        due: None,
        start: None,
        extras: BTreeMap::new(),
    }
}

const SURVIVOR: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Live\",\n  depends-on: (link(\"gone.typ\"),),\n)\n\n== Summary\n\nBody.\n";

#[test]
fn plan_tombstones_removed_and_rewrites_survivor() {
    let (_dir, root) = repo_with(&[("gone.typ", "gone body\n"), ("live.typ", SURVIVOR)]);
    let tasks = vec![
        task("gone", "gone.typ", Some("done")),
        task("live", "live.typ", None),
    ];

    let plan = plan(&tasks, &["done".to_owned()], &root, &root).unwrap();

    assert_eq!(plan.deletions.len(), 1);
    assert_eq!(plan.deletions[0].rel, PathBuf::from("gone.typ"));
    assert_eq!(plan.rewrites.len(), 1);
    let rewrite = &plan.rewrites[0];
    assert_eq!(rewrite.links, 1);
    assert!(rewrite.contents.contains("swh:1:rev:"));
    assert!(rewrite.contents.contains("path=gone.typ"));
    assert!(!rewrite.contents.contains("link(\"gone.typ\")"));
}

#[test]
fn apply_writes_and_deletes_without_committing() {
    let (_dir, root) = repo_with(&[("gone.typ", "gone\n"), ("live.typ", SURVIVOR)]);
    let tasks = vec![
        task("gone", "gone.typ", Some("done")),
        task("live", "live.typ", None),
    ];
    let plan = plan(&tasks, &["done".to_owned()], &root, &root).unwrap();
    apply(&plan).unwrap();

    assert!(!root.join("gone.typ").exists());
    let live = std::fs::read_to_string(root.join("live.typ")).unwrap();
    assert!(live.contains("swh:1:rev:"));
    // cleanup must not commit: the tree is left dirty for the user to review.
    assert!(!git::is_clean(&root).unwrap());
}

#[test]
fn empty_plan_when_nothing_terminal() {
    let (_dir, root) = repo_with(&[("live.typ", SURVIVOR)]);
    let tasks = vec![task("live", "live.typ", Some("wip"))];
    let plan = plan(&tasks, &["done".to_owned()], &root, &root).unwrap();
    assert_eq!(plan, Plan::default());
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

    let result = plan(&tasks, &["done".to_owned()], &root, &root);
    assert!(result.is_err());
    assert!(root.join("gone.typ").exists());
    assert!(root.join("orphan.typ").exists());
}

#[test]
fn is_clean_tracks_working_tree() {
    let (_dir, root) = repo_with(&[("a.typ", "a\n")]);
    assert!(git::is_clean(&root).unwrap());
    std::fs::write(root.join("a.typ"), "changed\n").unwrap();
    assert!(!git::is_clean(&root).unwrap());
}
