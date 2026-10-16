#![allow(clippy::unwrap_used)]

use crate::test_support::tempdir;
use std::path::Path;

use super::{
    AllowError, allowed_in, commands, digest, held_back, hold, record_in, record_path, repo_key,
    take_held,
};
use crate::config::Config;
use crate::waits::Checker;

fn config(runs: &[&str]) -> Config {
    let mut config = Config::default();
    config.tasks.waits = runs
        .iter()
        .enumerate()
        .map(|(index, run)| Checker {
            name: format!("c{index}"),
            run: (*run).to_owned(),
        })
        .collect();
    config
}

fn lines(runs: &[&str]) -> Vec<String> {
    runs.iter().map(|run| (*run).to_owned()).collect()
}

#[test]
fn the_commands_are_every_run_line_sorted_once() {
    assert_eq!(commands(&config(&["b", "a", "b"])), lines(&["a", "b"]));
    assert!(commands(&Config::default()).is_empty());
}

#[test]
fn the_commands_take_each_flip_run_line_beside_the_checkers() {
    let config = crate::parse_config(
        "[[tasks.waits]]\nname = \"up\"\nrun = \"b\"\n\
         [[on.flip]]\nwhen = ''\nactions = [{ run = \"c\", gate = true }, { show = \"x\" }]\n\
         [[on.flip]]\nwhen = ''\nactions = [{ run = \"b\" }, { gate = true }, { run = \"a\" }]\n",
    )
    .unwrap();

    assert_eq!(commands(&config), lines(&["a", "b", "c"]));
}

#[test]
fn taking_a_projects_held_commands_leaves_the_others_held() {
    hold(
        Path::new("taken"),
        &lines(&["held-taken-b", "held-taken-a"]),
    );
    hold(Path::new("kept"), &lines(&["held-kept"]));

    assert_eq!(
        take_held(Path::new("taken")),
        ["held-taken-a", "held-taken-b"]
    );

    let held = held_back();
    assert!(held.contains(&"held-kept".to_owned()));
    assert!(!held.iter().any(|line| line.starts_with("held-taken-")));
    assert!(take_held(Path::new("taken")).is_empty());
}

#[test]
fn a_held_command_is_listed_once() {
    hold(Path::new("one"), &lines(&["held-once-z", "held-once-y"]));
    hold(Path::new("two"), &lines(&["held-once-z"]));

    let held = held_back();

    assert_eq!(
        held.iter()
            .filter(|line| line.starts_with("held-once-"))
            .collect::<Vec<_>>(),
        ["held-once-y", "held-once-z"]
    );
}

#[test]
fn the_digest_changes_with_any_line_and_cannot_be_forged_by_joining_two() {
    let one = digest(&lines(&["a b"]));
    assert_eq!(one.len(), 40);
    assert_ne!(one, digest(&lines(&["a c"])));
    assert_ne!(one, digest(&lines(&["a", "b"])));
    assert_ne!(digest(&lines(&["ab"])), digest(&lines(&["a", "b"])));
}

#[test]
fn outside_git_the_key_is_the_root() {
    let dir = tempdir();

    assert_eq!(repo_key(dir.path()), dir.path().canonicalize().unwrap());
}

#[test]
fn inside_git_every_worktree_shares_the_common_dir_as_key() {
    let dir = tempdir();
    let main = dir.path().join("main");
    std::fs::create_dir_all(main.join("sub")).unwrap();
    crate::test_support::git_init(&main);
    crate::test_support::git_as_tester(&main, &["commit", "-q", "--allow-empty", "-m", "seed"]);
    let sibling = dir.path().join("sibling");
    crate::test_support::git_as_tester(
        &main,
        &["worktree", "add", "-q", sibling.to_str().unwrap()],
    );

    let key = repo_key(&main);

    assert_eq!(key, main.join(".git").canonicalize().unwrap());
    assert_eq!(repo_key(&main.join("sub")), key);
    assert_eq!(repo_key(&sibling), key);
}

#[test]
fn a_recorded_set_is_allowed_and_a_changed_one_is_not() {
    let dir = tempdir();
    let home = dir.path().join("home");
    let root = dir.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    let recorded = lines(&["scripts/locked {id}"]);

    assert!(!allowed_in(Some(&home), &root, &recorded));
    let path = record_in(Some(&home), &root, &recorded).unwrap();

    assert_eq!(path, record_path(&home, &repo_key(&root)));
    assert!(path.starts_with(home.join("mindtape").join("allowed")));
    assert!(allowed_in(Some(&home), &root, &recorded));
    assert!(!allowed_in(
        Some(&home),
        &root,
        &lines(&["scripts/other {id}"])
    ));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.ends_with("# scripts/locked {id}\n"), "{text}");
}

#[test]
fn no_commands_need_no_approval_and_no_data_home_allows_nothing() {
    let root = Path::new("/no/such/root");

    assert!(allowed_in(None, root, &[]));
    assert!(!allowed_in(None, root, &lines(&["a"])));
    assert!(matches!(
        record_in(None, root, &lines(&["a"])),
        Err(AllowError::NoDataHome)
    ));
}

#[test]
fn record_names_the_file_it_could_not_write() {
    let dir = tempdir();
    let home = dir.path().join("file");
    std::fs::write(&home, "").unwrap();

    let err = record_in(Some(&home), dir.path(), &lines(&["a"])).unwrap_err();

    assert!(matches!(err, AllowError::Write { .. }), "{err}");
    assert!(err.to_string().starts_with("can't write "), "{err}");
}
