#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tasks::{CommitStates, StateLoader, StatePlan};
use super::{ChangedFiles, CommitRecord, parse_message};
use crate::config::{Config, parse_config};
use crate::test_support::{commit_files, git_as_tester as git, git_tempdir, write_file};
use std::path::Path;

/// A raw task marker needing no prelude package, titled `title`.
fn marker(title: &str) -> String {
    format!("#metadata((marker: \"mindtape.task\", fields: (title: \"{title}\"), extras: (:)))\n")
}

/// Load the index plan over `config`, bounded to `root` so no developer
/// config layer applies.
fn load_index(root: &Path, config: &Config) -> CommitStates {
    let plan = StatePlan::index(root).unwrap();
    let mut loader =
        StateLoader::with_layers(root, &[], config, (Some(root.to_path_buf()), None, None));
    loader.load(&plan).unwrap()
}

/// The task ids a state holds.
fn ids(tasks: &[crate::model::Task]) -> Vec<&str> {
    tasks.iter().map(|task| task.id.as_str()).collect()
}

#[test]
fn a_state_carries_the_config_and_tasks_of_its_own_tree() {
    let dir = git_tempdir();
    let root = dir.path();
    let old = "[tasks]\nroots = [\"tasks\"]\n\n\
               [commits]\nexternal-refs = [\"^OLD-[0-9]+$\"]\n";
    let old_marker = marker("Old");
    commit_files(
        root,
        &[
            (".mindtape/config.toml", old),
            ("tasks/old.typ", &old_marker),
        ],
        "old",
    );

    // The index carries a new config and a task the working tree lacks.
    let new = "[tasks]\nroots = [\"tasks\"]\n\n\
               [commits]\nexternal-refs = [\"^NEW-[0-9]+$\"]\n";
    write_file(root, ".mindtape/config.toml", new);
    write_file(root, "tasks/new.typ", marker("New"));
    git(root, &["rm", "-q", "tasks/old.typ"]);
    git(root, &["add", "-A"]);
    write_file(root, ".mindtape/config.toml", old);
    std::fs::remove_file(root.join("tasks/new.typ")).unwrap();

    let config = parse_config(old).unwrap();
    let states = load_index(root, &config);
    assert_eq!(ids(&states.head.tasks), ["old"]);
    assert_eq!(ids(states.staged_tasks()), ["new"]);
    assert_eq!(states.head_config().commits.external_refs, ["^OLD-[0-9]+$"]);
    assert_eq!(
        states.staged_config().commits.external_refs,
        ["^NEW-[0-9]+$"]
    );
    assert_eq!(states.worktree_config(), Some(&config));
    assert!(
        states.staged.errors.is_empty(),
        "{:?}",
        states.staged.errors
    );

    // A record built the way the CLI builds it reads that same state.
    let message = parse_message("feat: x\n\nRefs: new NEW-7\n", "Refs");
    let (record, problems) = CommitRecord::build(
        &message,
        &ChangedFiles::default(),
        states.staged_tasks(),
        states.staged_config(),
        root,
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(record.refs, ["new", "NEW-7"]);
}

#[test]
fn a_broken_state_config_keeps_its_error_and_falls_back_to_the_working_config() {
    let dir = git_tempdir();
    let root = dir.path();
    let task = marker("Broken");
    commit_files(
        root,
        &[(".mindtape/config.toml", "[typst"), ("tasks/a.typ", &task)],
        "broken config",
    );
    let config = parse_config(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [commits]\nexternal-refs = [\"^W-[0-9]+$\"]\n",
    )
    .unwrap();
    let states = load_index(root, &config);
    assert_eq!(states.staged_config(), &config);
    assert_eq!(ids(states.staged_tasks()), ["a"]);
    let error = states
        .staged
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
