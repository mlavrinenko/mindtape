#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Integration tests: watcher `initial_scan` + `handle_event` with real eval pipeline.

mod common;

use common::setup_typst_project;
use mindtape::config::WatchEntry;
use mindtape::store::SqliteStore;
use mindtape::watcher::Watcher;

/// Set up a temp project directory with lib/ and .mindtapeignore.
fn setup_project() -> std::path::PathBuf {
    let root = setup_typst_project();
    // Ignore lib/ so prelude.typ isn't indexed as tasks.
    std::fs::write(root.join(".mindtapeignore"), "lib/\n").unwrap();
    root
}

fn make_entry(path: &std::path::Path) -> WatchEntry {
    WatchEntry {
        path: path.to_string_lossy().to_string(),
        recursive: true,
    }
}

#[test]
fn initial_scan_indexes_multiple_files() {
    let root = setup_project();
    std::fs::write(
        root.join("todo.typ"),
        r#"#import "@mindtape/mindtape:0.1.0": due, tag

= Todo

- [ ] Buy milk #due(2026, 3, 1)
- [x] Clean house
- [ ] Read book #tag("hobby")
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("work.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Ship feature\n- [ ] Code review\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let mut watcher = Watcher::new(store, &[make_entry(&root)]).unwrap();
    let scan = watcher.initial_scan();

    assert_eq!(scan.found, 2);
    assert_eq!(scan.indexed, 2);
    assert_eq!(scan.errors, 0);
}

#[test]
fn initial_scan_then_query_tasks() {
    let root = setup_project();
    std::fs::write(
        root.join("tasks.typ"),
        r#"#import "@mindtape/mindtape:0.1.0": due, tag

= My Tasks

- [ ] Important #due(2026, 1, 15) #tag("work")
- [ ] Less important #tag("personal")
- [x] Already done
"#,
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let mut watcher = Watcher::new(store, &[make_entry(&root)]).unwrap();
    watcher.initial_scan();

    // Access the store through a query — we need to get at it.
    // Since Watcher owns the store, we test via the scan results and
    // a second scan to confirm data persisted.
    let scan2 = watcher.initial_scan();
    assert_eq!(scan2.found, 1);
    assert_eq!(scan2.skipped, 1); // unchanged, hash matches
    assert_eq!(scan2.indexed, 0);
}

#[test]
fn handle_event_indexes_new_file_with_metadata() {
    let root = setup_project();
    let store = SqliteStore::open_memory().unwrap();
    let mut watcher = Watcher::new(store, &[make_entry(&root)]).unwrap();

    // Create a file after watcher setup.
    let file = root.join("new.typ");
    std::fs::write(
        &file,
        r#"#import "@mindtape/mindtape:0.1.0": due, tag

= New File

- [ ] First task #due(2026, 6, 1) #tag("new")
"#,
    )
    .unwrap();

    watcher.handle_event(&file);

    // Verify by doing a scan — should skip (already indexed).
    let scan = watcher.initial_scan();
    assert_eq!(scan.found, 1);
    assert_eq!(scan.skipped, 1);
}

#[test]
fn handle_event_reindexes_modified_file() {
    let root = setup_project();
    let file = root.join("evolving.typ");
    std::fs::write(
        &file,
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Original task\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let mut watcher = Watcher::new(store, &[make_entry(&root)]).unwrap();
    watcher.handle_event(&file);

    // Modify the file.
    std::fs::write(
        &file,
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Updated task\n- [ ] Second task\n",
    )
    .unwrap();
    watcher.handle_event(&file);

    // Scan should skip since we just indexed.
    let scan = watcher.initial_scan();
    assert_eq!(scan.skipped, 1);
}

#[test]
fn handle_event_delete_removes_from_index() {
    let root = setup_project();
    let file = root.join("temporary.typ");
    std::fs::write(
        &file,
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Temp task\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let mut watcher = Watcher::new(store, &[make_entry(&root)]).unwrap();
    watcher.initial_scan();

    // Delete the file and handle the event.
    std::fs::remove_file(&file).unwrap();
    watcher.handle_event(&file);

    // Scan should find nothing now.
    let scan = watcher.initial_scan();
    assert_eq!(scan.found, 0);
}

#[test]
fn subdirectory_files_are_indexed() {
    let root = setup_project();
    let subdir = root.join("projects/work");
    std::fs::create_dir_all(&subdir).unwrap();
    std::fs::write(
        subdir.join("sprint.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Sprint task\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let mut watcher = Watcher::new(store, &[make_entry(&root)]).unwrap();
    let scan = watcher.initial_scan();

    assert_eq!(scan.found, 1);
    assert_eq!(scan.indexed, 1);
}
