#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Integration tests: start/rank filter queries.

mod common;

use std::path::PathBuf;

use common::setup_typst_project;
use mindtape::store::{index_file, SqliteStore, Store, TaskFilter};
use mindtape::world::MindTapeWorld;

fn setup() -> (SqliteStore, PathBuf) {
    let root = setup_typst_project();
    let store = SqliteStore::open_memory().unwrap();
    (store, root)
}

fn add_and_index(store: &mut SqliteStore, root: &std::path::Path, rel: &str, content: &str) {
    let file = root.join(rel);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&file, content).unwrap();
    let world = MindTapeWorld::new(&file).unwrap();
    index_file(store, &world, &file, Some(root)).unwrap();
}

// --- start date filters ---

#[test]
fn list_with_start_before() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, start

- [ ] Early start #start(2026, 1, 15) #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Late start #start(2026, 6, 1) #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] No start #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            start_before: Some("2026-03-01".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Early start");
}

#[test]
fn list_with_start_after() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, start

- [ ] Early start #start(2026, 1, 15) #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Late start #start(2026, 6, 1) #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] No start #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            start_after: Some("2026-03-01".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Late start");
}

#[test]
fn list_with_start_range() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, start

- [ ] Early #start(2026, 1, 15) #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Middle #start(2026, 3, 1) #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Late #start(2026, 6, 1) #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            start_after: Some("2026-02-01".to_string()),
            start_before: Some("2026-04-01".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Middle");
}

// --- rank filters ---

#[test]
fn list_with_rank_min() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, rank, high, low

- [ ] High task #high #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Low task #low #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Custom rank #rank(75) #id("019c5b98-d10a-7710-8679-bda520780ee9")
- [ ] No rank #id("019c5b99-a9a0-7853-8b1a-72ec1d8bda37")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            rank_min: Some(50),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 2);
    let titles: Vec<&str> = views.iter().map(|v| v.title.as_str()).collect();
    assert!(titles.contains(&"High task"));
    assert!(titles.contains(&"Custom rank"));
}

#[test]
fn list_with_rank_max() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, rank, high, low

- [ ] High task #high #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Low task #low #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] No rank #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            rank_max: Some(50),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Low task");
}

#[test]
fn list_with_rank_range() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, rank

- [ ] Rank 10 #rank(10) #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Rank 50 #rank(50) #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Rank 100 #rank(100) #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            rank_min: Some(25),
            rank_max: Some(75),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Rank 50");
}
