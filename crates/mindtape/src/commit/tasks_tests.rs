#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::test_support::commit_files;
use crate::test_support::git_as_tester as git;
use crate::test_support::git_tempdir;
use crate::test_support::write_file;

use super::{CommitStates, StatePlan};
use crate::config::parse_config;

/// A raw task marker needing no prelude package.
const MARKER: &str =
    "#metadata((marker: \"mindtape.task\", fields: (title: \"Build\"), extras: (:)))\n";

/// A repository with `kept.typ` and `moved.typ` committed, then `kept.typ`
/// edited and staged, `moved.typ` renamed and staged, and `loose.typ` left
/// untracked.
fn repo() -> tempfile::TempDir {
    let dir = git_tempdir();
    let root = dir.path();
    write_file(root, "kept.typ", "old\n");
    write_file(root, "moved.typ", "same\n");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "seed"]);
    write_file(root, "kept.typ", "new\n");
    git(root, &["mv", "moved.typ", "renamed.typ"]);
    git(root, &["add", "kept.typ"]);
    write_file(root, "loose.typ", "x\n");
    dir
}

#[test]
fn the_index_plan_names_every_side_of_each_change() {
    let dir = repo();
    let plan = StatePlan::index(dir.path()).unwrap();
    let mut head = plan.head.differs.clone();
    head.sort();
    assert_eq!(head, ["kept.typ", "loose.typ", "moved.typ", "renamed.typ"]);
    assert_eq!(plan.staged.differs, ["loose.typ"]);
    assert_eq!(
        (plan.head.rev.as_str(), plan.staged.rev.as_str()),
        ("HEAD", "")
    );
    assert!(plan.worktree);
    assert_eq!(plan.root_prefix, "");
}

#[test]
fn a_plan_before_the_first_commit_reads_the_empty_tree() {
    let dir = git_tempdir();
    let root = dir.path();
    write_file(root, "first.typ", "x\n");
    git(root, &["add", "first.typ"]);
    let plan = StatePlan::index(root).unwrap();
    assert_ne!(plan.head.rev, "HEAD");
    assert!(!plan.head.rev.is_empty(), "the empty tree has an id");
}

/// A prelude stub that ignores its arguments and always reports `title`.
fn titled_prelude(title: &str) -> String {
    format!(
        "#let task(body, ..meta) = [#metadata((marker: \"mindtape.task\", \
         fields: (title: \"{title}\"), extras: (:))) <mindtape>]\n"
    )
}

/// A task importing the pinned package at 0.3.0.
const PINNED_TASK: &str = "#import \"@local/mindtape:0.3.0\": *\n\n\
                           #show: task.with(title: \"Write the parser\")\n";

/// A plan with `head` and `staged` reading the named revisions, both with no
/// changed paths.
fn revisions(head: &str, staged: &str) -> StatePlan {
    StatePlan {
        root_prefix: String::new(),
        head: super::StateSource {
            rev: head.to_owned(),
            differs: Vec::new(),
        },
        staged: super::StateSource {
            rev: staged.to_owned(),
            differs: Vec::new(),
        },
        worktree: false,
    }
}

#[test]
fn a_state_without_a_config_uses_the_working_trees_roots() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(root, &[("tasks/a.typ", MARKER)], "no config");
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    let plan = revisions("HEAD", "HEAD");
    let states = CommitStates::load(root, &[], &config, &plan).unwrap();
    assert_eq!(states.head.tasks.len(), 1);
    assert_eq!(states.head.tasks[0].id, "a");
    assert!(states.head.errors.is_empty(), "{:?}", states.head.errors);
}

#[test]
fn a_malformed_state_config_falls_back_and_is_reported() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[(".mindtape/config.toml", "[typst"), ("tasks/a.typ", MARKER)],
        "broken config",
    );
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    let plan = revisions("HEAD", "HEAD");
    let states = CommitStates::load(root, &[], &config, &plan).unwrap();
    assert_eq!(
        states.head.tasks.len(),
        1,
        "the working roots still load it"
    );
    let error = states
        .head
        .errors
        .first()
        .expect("the broken config is reported");
    assert_eq!(
        error.file,
        std::path::PathBuf::from(".mindtape/config.toml")
    );
    assert!(
        error.message.contains("working tree's config"),
        "{}",
        error.message
    );
}

#[test]
fn a_state_loads_its_unchanged_task_against_its_own_pin() {
    let dir = git_tempdir();
    let root = dir.path();
    let manifest =
        "[package]\nname = \"mindtape\"\nversion = \"0.3.0\"\nentrypoint = \"prelude.typ\"\n";
    let old = titled_prelude("Old pin");
    let old_config = "[tasks]\nroots = [\"tasks\"]\n\n[[typst.local-packages]]\nname = \"mindtape\"\npath = \"old-lib\"\n";
    commit_files(
        root,
        &[
            ("old-lib/typst.toml", manifest),
            ("old-lib/prelude.typ", &old),
            (".mindtape/config.toml", old_config),
            ("tasks/parser.typ", PINNED_TASK),
        ],
        "old pin",
    );
    let new = titled_prelude("New pin");
    let new_config = "[tasks]\nroots = [\"tasks\"]\n\n[[typst.local-packages]]\nname = \"mindtape\"\npath = \"lib\"\n";
    commit_files(
        root,
        &[
            ("lib/typst.toml", manifest),
            ("lib/prelude.typ", &new),
            (".mindtape/config.toml", new_config),
        ],
        "move pin",
    );

    let config = parse_config(
        "[tasks]\nroots = [\"tasks\"]\n\n[[typst.local-packages]]\nname = \"mindtape\"\npath = \"lib\"\n",
    )
    .unwrap();
    let plan = revisions("HEAD~1", "HEAD");
    let states = CommitStates::load(root, &[], &config, &plan).unwrap();
    assert_eq!(states.head.tasks[0].title, "Old pin");
    assert_eq!(states.staged.tasks[0].title, "New pin");
    assert!(states.head.errors.is_empty(), "{:?}", states.head.errors);
    assert!(
        states.staged.errors.is_empty(),
        "{:?}",
        states.staged.errors
    );
}
