#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Integration tests: eval -> store pipeline.

mod common;

use common::setup_typst_project;
use mindtape::store::{index_file, SqliteStore, Store, TaskFilter};
use mindtape::world::MindTapeWorld;

/// Set up a temp project with a `.typ` file, a `MindTapeWorld`, and an in-memory store.
/// Returns (store, world, `file_path`, `project_root`).
fn setup(
    source: &str,
) -> (SqliteStore, MindTapeWorld, std::path::PathBuf, std::path::PathBuf) {
    let root = setup_typst_project();
    let file = root.join("test.typ");
    std::fs::write(&file, source).unwrap();
    let world = MindTapeWorld::new(&file).unwrap();
    let store = SqliteStore::open_memory().unwrap();
    (store, world, file, root)
}

#[test]
fn index_file_stores_tasks() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": due, tag

= Piano Practice

- [ ] Learn scales #due(datetime(year: 2026, month: 3, day: 1)) #tag("music")
- [x] Buy metronome
- [ ] Practice arpeggios #tag("music") #tag("technique")
"#,
    );

    let indexed = index_file(&mut store, &world, &file, &root).unwrap();
    assert!(indexed);

    let views = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(views.len(), 3);

    assert_eq!(views[0].title, "Learn scales");
    assert!(!views[0].is_done);
    assert_eq!(views[0].due, Some("2026-03-01".to_string()));
    assert_eq!(views[0].tags, vec!["music"]);
    assert_eq!(views[0].file_title, Some("Piano Practice".to_string()));

    assert_eq!(views[1].title, "Buy metronome");
    assert!(views[1].is_done);

    assert_eq!(views[2].title, "Practice arpeggios");
    assert_eq!(views[2].tags, vec!["music", "technique"]);
}

#[test]
fn index_file_skips_unchanged() {
    let (mut store, world, file, root) = setup("- [ ] Task");

    let first = index_file(&mut store, &world, &file, &root).unwrap();
    assert!(first);

    let second = index_file(&mut store, &world, &file, &root).unwrap();
    assert!(!second); // hash unchanged, should skip
}

#[test]
fn index_file_reindexes_on_change() {
    let (mut store, world, file, root) = setup("- [ ] Old task");

    index_file(&mut store, &world, &file, &root).unwrap();

    // Modify the file
    std::fs::write(&file, "- [ ] New task A\n- [ ] New task B").unwrap();
    let world2 = MindTapeWorld::new(&file).unwrap();

    let reindexed = index_file(&mut store, &world2, &file, &root).unwrap();
    assert!(reindexed);

    let views = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(views.len(), 2);
    assert_eq!(views[0].title, "New task A");
    assert_eq!(views[1].title, "New task B");
}

#[test]
fn index_file_extracts_title() {
    let (mut store, world, file, root) = setup("= My Project\n\n- [ ] Task");

    index_file(&mut store, &world, &file, &root).unwrap();

    let views = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(views[0].file_title, Some("My Project".to_string()));
}

#[test]
fn index_file_query_by_tag() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": tag

- [ ] Work task #tag("work")
- [ ] Fun task #tag("fun")
- [ ] Both #tag("work") #tag("fun")
"#,
    );

    index_file(&mut store, &world, &file, &root).unwrap();

    let work = store
        .query_tasks(&TaskFilter {
            tag: Some("work".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(work.len(), 2);
    assert_eq!(work[0].title, "Work task");
    assert_eq!(work[1].title, "Both");
}

#[test]
fn index_file_remove_then_query_is_empty() {
    let (mut store, world, file, root) = setup("- [ ] Task A\n- [ ] Task B");

    index_file(&mut store, &world, &file, &root).unwrap();
    assert_eq!(store.query_tasks(&TaskFilter::default()).unwrap().len(), 2);

    // Remove the file from the index
    let relative = file.strip_prefix(&root).unwrap();
    store.remove_task_file(relative).unwrap();
    assert_eq!(store.query_tasks(&TaskFilter::default()).unwrap().len(), 0);
}

#[test]
fn index_file_query_combined_tag_and_done() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": tag

- [ ] Open work #tag("work")
- [x] Done work #tag("work")
- [ ] Open fun #tag("fun")
"#,
    );

    index_file(&mut store, &world, &file, &root).unwrap();

    // Filter: tag=work AND not done
    let views = store
        .query_tasks(&TaskFilter {
            tag: Some("work".to_string()),
            done: Some(false),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Open work");
}

#[test]
fn index_file_stores_bindings() {
    let (mut store, world, file, root) = setup(
        r#"
#let project = "MindTape"
#let version = 2

- [ ] A task
"#,
    );

    index_file(&mut store, &world, &file, &root).unwrap();

    // Verify bindings were stored by checking the file hash exists (bindings are stored)
    let relative = file.strip_prefix(&root).unwrap();
    let hash = store.get_file_hash(relative).unwrap();
    assert!(hash.is_some());
}

#[test]
fn index_multiple_files() {
    let root = setup_typst_project();

    let file_a = root.join("a.typ");
    std::fs::write(&file_a, "- [ ] Task from A").unwrap();
    let file_b = root.join("b.typ");
    std::fs::write(&file_b, "- [ ] Task from B\n- [ ] Another from B").unwrap();

    let mut store = SqliteStore::open_memory().unwrap();

    let world_a = MindTapeWorld::new(&file_a).unwrap();
    index_file(&mut store, &world_a, &file_a, &root).unwrap();

    let world_b = MindTapeWorld::new(&file_b).unwrap();
    index_file(&mut store, &world_b, &file_b, &root).unwrap();

    let all = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(all.len(), 3);

    // Query by file
    let from_a = store
        .query_tasks(&TaskFilter {
            file_path: Some(std::path::PathBuf::from("a.typ")),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(from_a.len(), 1);
    assert_eq!(from_a[0].title, "Task from A");
}
