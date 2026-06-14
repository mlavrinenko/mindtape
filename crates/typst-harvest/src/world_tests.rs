#![allow(clippy::unwrap_used)]

use std::path::Path;

use typst::World;

use super::{HarvestWorld, find_project_root};

#[test]
fn root_marker_priority() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".mindtape")).unwrap();
    assert_eq!(find_project_root(dir.path()), dir.path());
}

#[test]
fn root_walks_up_to_git() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    let child = dir.path().join("a").join("b");
    std::fs::create_dir_all(&child).unwrap();
    assert_eq!(find_project_root(&child), dir.path());
}

#[test]
fn new_reads_main_source() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".mindtape")).unwrap();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, "= Hello").unwrap();

    let world = HarvestWorld::new(&file).unwrap();
    let source = world.source(world.main()).unwrap();
    assert!(source.text().contains("Hello"));
}

#[test]
fn with_root_rejects_outside_file() {
    let outer = tempfile::tempdir().unwrap();
    let inner = tempfile::tempdir().unwrap();
    let file = outer.path().join("task.typ");
    std::fs::write(&file, "= x").unwrap();
    // file is not under `inner`, so construction must fail.
    assert!(HarvestWorld::with_root(&file, inner.path()).is_err());
}

#[test]
fn new_rejects_missing_file() {
    assert!(HarvestWorld::new(Path::new("/nonexistent/x.typ")).is_err());
}
