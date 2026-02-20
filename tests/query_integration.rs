#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Integration tests: query commands (`list_files`, `get_stats`, folder filter).

mod common;

use std::path::PathBuf;

use common::setup_typst_project;
use mindtape::store::{index_file, SqliteStore, Store, TaskFilter};
use mindtape::world::MindTapeWorld;

/// Set up a temp project with `lib/` and return (store, project root).
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
    index_file(store, &world, &file, Some(root)).unwrap();
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
    // Paths are now absolute.
    assert_eq!(files[0].file_path, root.join("notes/work.typ"));
    assert_eq!(files[0].task_count, 1);
    assert_eq!(files[1].file_path, root.join("todo.typ"));
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

    // Folder filter uses absolute path prefix.
    let notes_folder = root.join("notes/");
    let notes = store
        .query_tasks(&TaskFilter {
            folder: Some(notes_folder),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(notes.len(), 2);

    let other_folder = root.join("other/");
    let other = store
        .query_tasks(&TaskFilter {
            folder: Some(other_folder),
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
            tags: vec!["work".to_string()],
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

    // Remove using absolute path.
    store.remove_task_file(&root.join("a.typ")).unwrap();

    let files = store.list_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_path, root.join("b.typ"));
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

    let notes_folder = root.join("notes/");
    let views = store
        .query_tasks(&TaskFilter {
            folder: Some(notes_folder),
            done: Some(false),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Open note");
}

// --- FTS5 search & new filters (end-to-end) ---

#[test]
fn search_tasks_by_title() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id

- [ ] Buy groceries #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Write documentation #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [x] Fix parser bug #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            search: Some("groceries".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Buy groceries");
}

#[test]
fn search_tasks_by_milestone() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id

= Sprint 3

- [ ] Deploy service #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")

= Backlog

- [ ] Research options #id("019c5b97-9239-7270-b7d7-2a50806912b3")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            search: Some("sprint".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Deploy service");
}

#[test]
fn list_with_multiple_tags() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, tag

- [ ] Both tags #tag("work") #tag("urgent") #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Only work #tag("work") #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Only urgent #tag("urgent") #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            tags: vec!["work".to_string(), "urgent".to_string()],
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Both tags");
}

#[test]
fn list_with_due_range() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id, due

- [ ] Early #due(datetime(year: 2026, month: 1, day: 15)) #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
- [ ] Middle #due(datetime(year: 2026, month: 3, day: 1)) #id("019c5b97-9239-7270-b7d7-2a50806912b3")
- [ ] Late #due(datetime(year: 2026, month: 6, day: 1)) #id("019c5b98-d10a-7710-8679-bda520780ee9")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            due_after: Some("2026-02-01".to_string()),
            due_before: Some("2026-04-01".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Middle");
}

#[test]
fn query_tasks_by_watch_root() {
    let (mut store, root) = setup();

    // Index files under the main root.
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Root task #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")
"#,
    );

    // Create a second "watch root" directory and index a file under it.
    let root2 = root.join("second-root");
    std::fs::create_dir_all(&root2).unwrap();
    // Copy lib/ so the Typst import resolves.
    let lib_dst = root2.join("lib");
    std::fs::create_dir_all(&lib_dst).unwrap();
    for entry in std::fs::read_dir(root.join("lib")).unwrap() {
        let lib_entry = entry.unwrap();
        std::fs::copy(lib_entry.path(), lib_dst.join(lib_entry.file_name())).unwrap();
    }
    let file2 = root2.join("tasks.typ");
    std::fs::write(
        &file2,
        r#"#import "@mindtape/mindtape:0.1.0": id
- [ ] Second root task #id("019c5b97-9239-7270-b7d7-2a50806912b3")
"#,
    )
    .unwrap();
    let world2 = MindTapeWorld::new(&file2).unwrap();
    index_file(&mut store, &world2, &file2, Some(&root2)).unwrap();

    // Filter by the first watch root.
    let first = store
        .query_tasks(&TaskFilter {
            watch_root: Some(root.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].title, "Root task");

    // Filter by the second watch root.
    let second = store
        .query_tasks(&TaskFilter {
            watch_root: Some(root2),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].title, "Second root task");

    // No filter returns both.
    let all = store.query_tasks(&TaskFilter::default()).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn list_with_milestone_filter() {
    let (mut store, root) = setup();
    add_and_index(
        &mut store,
        &root,
        "todo.typ",
        r#"#import "@mindtape/mindtape:0.1.0": id

= Engineering
== Backend

- [ ] Fix API #id("019c5b9b-7317-77b1-bf52-ce7a298cfcad")

= Design

- [ ] Update mockups #id("019c5b97-9239-7270-b7d7-2a50806912b3")
"#,
    );

    let views = store
        .query_tasks(&TaskFilter {
            milestone: Some("Backend".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Fix API");
}
