#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Integration tests: query commands (`list_files`, `get_stats`, folder filter).

mod common;

use std::path::PathBuf;

use common::setup_typst_project;
use mindtape::store::{index_file, SqliteStore, Store, TaskFilter};
use mindtape::world::MindTapeWorld;

/// Set up a temp project with lib/ and return (store, `project_root`).
/// Caller adds .typ files and indexes them.
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
    index_file(store, &world, &file, root).unwrap();
}

// --- list_files ---

#[test]
fn list_files_after_indexing() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id

= My Tasks

- [ ] Buy milk #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [x] Done #id("019c5b97-9239-7270-b7d7-2a50806912b3")
"#,
    );
    add_and_index(
        &mut store,
        &root,
        "notes/work.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Write report #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let files = store.list_files().unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].relative_path, PathBuf::from("notes/work.typ"));
    assert_eq!(files[0].task_count, 1);
    assert_eq!(files[1].relative_path, PathBuf::from("todo.typ"));
    assert_eq!(files[1].task_count, 2);
    assert_eq!(files[1].title, Some("My Tasks".to_string()));
}

// --- get_stats ---

#[test]
fn stats_after_indexing() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Pending A #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [x] Done B #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Pending C #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );
    add_and_index(
        &mut store,
        &root,
        "other.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [x] Done D #id("019c5b99-a9a0-7853-8b1a-72ec1d8bda37")
"#,
    );

    let stats = store.get_stats().unwrap();
    assert_eq!(stats.file_count, 2);
    assert_eq!(stats.task_count, 4);
    assert_eq!(stats.done_count, 2);
    assert_eq!(stats.pending_count, 2);
    assert!(stats.last_updated.is_some());
}

// --- folder filter ---

#[test]
fn query_tasks_by_folder() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "notes/todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Note task #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
"#,
    );
    add_and_index(
        &mut store,
        &root,
        "notes/deep/nested.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Deep task #id("019c5b97-9239-7270-b7d7-2a50806912b3")
"#,
    );
    add_and_index(
        &mut store,
        &root,
        "other/misc.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Other task #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let notes = store
        .query_tasks(&TaskFilter {
            folder: Some(PathBuf::from("notes/")),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(notes.len(), 2);
    assert!(notes.iter().all(|t| t.file_path.starts_with("notes/")));

    let other = store
        .query_tasks(&TaskFilter {
            folder: Some(PathBuf::from("other/")),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].title, "Other task");
}

// --- list with combined filters ---

#[test]
fn list_done_tasks_by_tag() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, tag

- [ ] Open work #tag("work") #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [x] Done work #tag("work") #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [x] Done fun #tag("fun") #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            done: Some(true),
            tag: Some("work".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Done work");
}

// --- stats on empty database ---

#[test]
fn stats_empty_db() {
    let (store, _root) = setup();
    let stats = store.get_stats().unwrap();
    assert_eq!(stats.file_count, 0);
    assert_eq!(stats.task_count, 0);
    assert!(stats.last_updated.is_none());
}

// --- list_files after file removal ---

#[test]
fn list_files_after_removal() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "a.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Task A #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
"#,
    );
    add_and_index(
        &mut store,
        &root,
        "b.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Task B #id("019c5b97-9239-7270-b7d7-2a50806912b3")
"#,
    );

    assert_eq!(store.list_files().unwrap().len(), 2);

    store
        .remove_task_file(std::path::Path::new("a.typ"))
        .unwrap();

    let files = store.list_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].relative_path, PathBuf::from("b.typ"));
}

// --- folder + done combined ---

#[test]
fn folder_and_status_combined() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "notes/todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Open note #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [x] Done note #id("019c5b97-9239-7270-b7d7-2a50806912b3")
"#,
    );
    add_and_index(
        &mut store,
        &root,
        "other/misc.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Open other #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            folder: Some(PathBuf::from("notes/")),
            done: Some(false),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Open note");
}
