#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};

use super::load_many;
use crate::loader::tests::TASK_BODY;
use crate::test_support::{tempdir, write_bare_project, write_file};

/// A bare project at `dir/name` holding one task named `stem`.
fn project(dir: &Path, name: &str, stem: &str) -> PathBuf {
    let root = dir.join(name);
    write_bare_project(&root);
    write_file(&root, format!("tasks/{stem}.typ"), TASK_BODY);
    root
}

#[test]
fn each_root_comes_back_in_the_order_given() {
    let dir = tempdir();
    let roots: Vec<PathBuf> = ["c", "a", "b"]
        .iter()
        .map(|name| project(dir.path(), name, &format!("task-{name}")))
        .collect();

    let loaded = load_many(&roots);

    let stems: Vec<String> = loaded
        .iter()
        .map(|member| member.as_ref().unwrap().outcome.tasks[0].id.clone())
        .collect();
    assert_eq!(stems, ["task-c", "task-a", "task-b"]);
}

#[test]
fn a_broken_config_fails_only_its_member() {
    let dir = tempdir();
    let good = project(dir.path(), "good", "fine");
    let broken = project(dir.path(), "broken", "lost");
    write_file(&broken, ".mindtape/config.toml", "[tasks\n");

    let loaded = load_many(&[broken, good]);

    assert!(loaded[0].is_err());
    assert_eq!(loaded[1].as_ref().unwrap().outcome.tasks[0].id, "fine");
}

#[test]
fn no_roots_load_nothing() {
    assert!(load_many(&[]).is_empty());
}
