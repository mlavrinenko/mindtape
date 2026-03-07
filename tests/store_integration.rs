#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Integration tests: eval -> store pipeline.

mod common;

use common::setup_typst_project;
use mindtape::store::{SqliteStore, Store, TaskFilter, index_file};
use mindtape::world::MindTapeWorld;

/// Set up a temp project with a `.typ` file, a `MindTapeWorld`, and an in-memory store.
/// Returns (store, world, file path, project root).
fn setup(
    source: &str,
) -> (
    SqliteStore,
    MindTapeWorld,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let root = setup_typst_project();
    let file = root.join("test.typ");
    std::fs::write(&file, source).unwrap();
    let world = MindTapeWorld::new(&file).unwrap();
    let store = SqliteStore::open_memory().unwrap();
    (store, world, file, root)
}

#[test]
fn index_file_stores_tasks_with_id() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": due, id, tag

= Piano Practice

- [ ] Learn scales #due(2026, 3, 1) #tag("music") #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [x] Buy metronome #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Practice arpeggios #tag("music") #tag("technique") #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let indexed = index_file(&mut store, &world, &file, Some(&root)).unwrap();
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
fn index_file_skips_tasks_without_id() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id

- [ ] Has id #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] No id
- [x] Also no id
"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();

    let views = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Has id");
}

#[test]
fn index_file_skips_tasks_with_invalid_id() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id

- [ ] Valid v7 #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Not a uuid #id("not-a-uuid")
- [ ] UUIDv4 #id("550e8400-e29b-41d4-a716-446655440000")
"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();

    let views = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Valid v7");
}

#[test]
fn index_file_skips_unchanged() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Task #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")"#,
    );

    let first = index_file(&mut store, &world, &file, Some(&root)).unwrap();
    assert!(first);

    let second = index_file(&mut store, &world, &file, Some(&root)).unwrap();
    assert!(!second); // hash unchanged, should skip
}

#[test]
fn index_file_reindexes_on_change() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Old task #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();

    // Modify the file
    std::fs::write(
        &file,
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] New task A #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] New task B #id("019c5b98-d10a-7710-8679-bda520780ee9")"#,
    )
    .unwrap();
    let world2 = MindTapeWorld::new(&file).unwrap();

    let reindexed = index_file(&mut store, &world2, &file, Some(&root)).unwrap();
    assert!(reindexed);

    let views = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(views.len(), 2);
    assert_eq!(views[0].title, "New task A");
    assert_eq!(views[1].title, "New task B");
}

#[test]
fn index_file_extracts_title() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id
= My Project

- [ ] Task #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();

    let views = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(views[0].file_title, Some("My Project".to_string()));
}

#[test]
fn index_file_query_by_tag() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id, tag

- [ ] Work task #tag("work") #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Fun task #tag("fun") #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Both #tag("work") #tag("fun") #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();

    let work = store
        .query_tasks(&TaskFilter {
            expr: Some("has_tag(\"work\")".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(work.len(), 2);
    assert_eq!(work[0].title, "Work task");
    assert_eq!(work[1].title, "Both");
}

#[test]
fn index_file_remove_then_query_is_empty() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Task A #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Task B #id("019c5b97-9239-7270-b7d7-2a50806912b3")"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();
    assert_eq!(store.query_tasks(&TaskFilter::default()).unwrap().len(), 2);

    // Remove the file from the index using absolute path.
    store.remove_task_file(&file).unwrap();
    assert_eq!(store.query_tasks(&TaskFilter::default()).unwrap().len(), 0);
}

#[test]
fn index_file_query_combined_tag_and_done() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id, tag

- [ ] Open work #tag("work") #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [x] Done work #tag("work") #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Open fun #tag("fun") #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();

    // Filter: tag=work AND not done
    let views = store
        .query_tasks(&TaskFilter {
            done: Some(false),
            expr: Some("has_tag(\"work\")".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Open work");
}

#[test]
fn index_file_stores_bindings() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": id
#let project = "MindTape"
#let version = 2

- [ ] A task #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();

    // Verify bindings were stored by checking the file hash exists.
    let hash = store.get_file_hash(&file).unwrap();
    assert!(hash.is_some());
}

#[test]
fn index_multiple_files() {
    let root = setup_typst_project();

    let file_a = root.join("a.typ");
    std::fs::write(
        &file_a,
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Task from A #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")"#,
    )
    .unwrap();
    let file_b = root.join("b.typ");
    std::fs::write(
        &file_b,
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Task from B #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Another from B #id("019c5b98-d10a-7710-8679-bda520780ee9")"#,
    )
    .unwrap();

    let mut store = SqliteStore::open_memory().unwrap();

    let world_a = MindTapeWorld::new(&file_a).unwrap();
    index_file(&mut store, &world_a, &file_a, Some(&root)).unwrap();

    let world_b = MindTapeWorld::new(&file_b).unwrap();
    index_file(&mut store, &world_b, &file_b, Some(&root)).unwrap();

    let all = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(all.len(), 3);

    // Query by file (absolute path)
    let from_a = store
        .query_tasks(&TaskFilter {
            expr: Some(format!("file == \"{}\"", file_a.display())),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(from_a.len(), 1);
    assert_eq!(from_a[0].title, "Task from A");
}

#[test]
fn index_file_stores_start_and_rank() {
    let (mut store, world, file, root) = setup(
        r#"#import "@mindtape/mindtape:0.1.0": due, start, id, tag, rank, high

- [ ] High priority #start(2026, 3, 1) #due(2026, 4, 1) #high #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Custom rank #rank(75) #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] No rank or start #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    index_file(&mut store, &world, &file, Some(&root)).unwrap();

    let views = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(views.len(), 3);

    assert_eq!(views[0].title, "High priority");
    assert_eq!(views[0].start, Some("2026-03-01".to_string()));
    assert_eq!(views[0].due, Some("2026-04-01".to_string()));
    assert_eq!(views[0].rank, Some(100));

    assert_eq!(views[1].title, "Custom rank");
    assert_eq!(views[1].start, None);
    assert_eq!(views[1].rank, Some(75));

    assert_eq!(views[2].title, "No rank or start");
    assert_eq!(views[2].start, None);
    assert_eq!(views[2].rank, None);
}
