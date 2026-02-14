//! Integration tests: query commands (list_files, get_stats, folder filter).

use std::path::PathBuf;

use mindtape::store::{index_file, SqliteStore, Store, TaskFilter};
use mindtape::world::MindTapeWorld;

/// Set up a temp project with lib/ and return (store, project_root).
/// Caller adds .typ files and indexes them.
fn setup() -> (SqliteStore, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.keep();
    std::fs::write(root.join("Cargo.toml"), "").unwrap();
    let lib_dir = root.join("lib");
    std::fs::create_dir(&lib_dir).unwrap();
    std::fs::write(
        lib_dir.join("prelude.typ"),
        r#"
#let due(date) = metadata(("due", date))
#let id(uuid) = metadata(("id", uuid))
#let tag(name) = metadata(("tag", name))
"#,
    )
    .unwrap();
    std::fs::write(
        lib_dir.join("typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.1.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
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
        "= My Tasks\n\n- [ ] Buy milk\n- [x] Done\n",
    );
    add_and_index(
        &mut store,
        &root,
        "notes/work.typ",
        "- [ ] Write report\n",
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
        "- [ ] Pending A\n- [x] Done B\n- [ ] Pending C\n",
    );
    add_and_index(
        &mut store,
        &root,
        "other.typ",
        "- [x] Done D\n",
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
        "- [ ] Note task\n",
    );
    add_and_index(
        &mut store,
        &root,
        "notes/deep/nested.typ",
        "- [ ] Deep task\n",
    );
    add_and_index(
        &mut store,
        &root,
        "other/misc.typ",
        "- [ ] Other task\n",
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
        r#"#import "@mindtape/mindtape:0.1.0": tag

- [ ] Open work #tag("work")
- [x] Done work #tag("work")
- [x] Done fun #tag("fun")
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
    add_and_index(&mut store, &root, "a.typ", "- [ ] Task A\n");
    add_and_index(&mut store, &root, "b.typ", "- [ ] Task B\n");

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
        "- [ ] Open note\n- [x] Done note\n",
    );
    add_and_index(
        &mut store,
        &root,
        "other/misc.typ",
        "- [ ] Open other\n",
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
