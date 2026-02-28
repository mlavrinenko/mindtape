use super::*;
use super::ignore::build_ignore;
use crate::store::SqliteStore;
use std::fs;

fn setup_watch_dir() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    // Create project root markers.
    fs::write(dir.path().join("Cargo.toml"), "").unwrap();
    fs::create_dir_all(dir.path().join("lib")).unwrap();
    fs::write(
        dir.path().join("lib/typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.1.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("lib/prelude.typ"),
        "#let due(date) = metadata((\"due\", date))\n#let id(uuid) = metadata((\"id\", uuid))\n#let tag(name) = metadata((\"tag\", name))\n",
    )
    .unwrap();
    // Ignore lib/ so prelude.typ doesn't get indexed as a task file.
    fs::write(dir.path().join(".mindtapeignore"), "lib/\n").unwrap();
    let dir_path = dir.path().to_path_buf();
    (dir, dir_path)
}

fn make_entry(path: &Path) -> WatchEntry {
    WatchEntry {
        path: path.to_string_lossy().to_string(),
        recursive: true,
    }
}

// --- is_typ_file ---

#[test]
fn is_typ_file_true_for_typ() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.typ");
    fs::write(&file, "").unwrap();
    assert!(is_typ_file(&file));
}

#[test]
fn is_typ_file_false_for_other_ext() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "").unwrap();
    assert!(!is_typ_file(&file));
}

#[test]
fn is_typ_file_false_for_directory() {
    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("subdir.typ");
    fs::create_dir(&subdir).unwrap();
    assert!(!is_typ_file(&subdir));
}

#[test]
fn is_typ_file_false_for_nonexistent() {
    assert!(!is_typ_file(Path::new("/nonexistent/file.typ")));
}

// --- is_inside_dotgit ---

#[test]
fn dotgit_path_is_detected() {
    assert!(is_inside_dotgit(Path::new("/home/user/repo/.git/HEAD")));
    assert!(is_inside_dotgit(Path::new("/repo/.git/refs/heads/main")));
    assert!(is_inside_dotgit(Path::new("/repo/.git/objects/pack")));
}

#[test]
fn non_dotgit_path_is_not_detected() {
    assert!(!is_inside_dotgit(Path::new("/home/user/repo/src/main.rs")));
    assert!(!is_inside_dotgit(Path::new("/home/user/.gitconfig")));
    assert!(!is_inside_dotgit(Path::new("/repo/todo.typ")));
}

// --- build_ignore ---

#[test]
fn build_ignore_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.typ");
    let ig = build_ignore(dir.path(), &file);
    // With no local ignore files, .typ files should not be ignored.
    assert!(!ig.matched("test.typ", false).is_ignore());
}

#[test]
fn build_ignore_with_mindtapeignore() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".mindtapeignore"), "*.bak\ndrafts/\n").unwrap();
    let file = dir.path().join("test.bak");
    let ig = build_ignore(dir.path(), &file);
    assert!(ig.matched("test.bak", false).is_ignore());
    assert!(!ig.matched("test.typ", false).is_ignore());
}

#[test]
fn build_ignore_with_gitignore() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".gitignore"), "*.draft.typ\nbuild/\n").unwrap();
    let file = dir.path().join("notes.draft.typ");
    let ig = build_ignore(dir.path(), &file);
    assert!(ig.matched("notes.draft.typ", false).is_ignore());
    assert!(ig.matched("build", true).is_ignore()); // directory pattern
    assert!(!ig.matched("todo.typ", false).is_ignore());
}

#[test]
fn build_ignore_combines_gitignore_and_mindtapeignore() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".gitignore"), "*.draft.typ\n").unwrap();
    fs::write(dir.path().join(".mindtapeignore"), "secret.typ\n").unwrap();
    let file = dir.path().join("notes.draft.typ");
    let ig = build_ignore(dir.path(), &file);
    // Both patterns should be respected.
    assert!(ig.matched("notes.draft.typ", false).is_ignore());
    assert!(ig.matched("secret.typ", false).is_ignore());
    assert!(!ig.matched("todo.typ", false).is_ignore());
}

#[test]
fn build_ignore_nested_gitignore() {
    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("sub");
    fs::create_dir(&subdir).unwrap();
    // Root .gitignore ignores *.bak, nested .gitignore ignores *.draft.typ.
    fs::write(dir.path().join(".gitignore"), "*.bak\n").unwrap();
    fs::write(subdir.join(".gitignore"), "*.draft.typ\n").unwrap();
    let file = subdir.join("notes.draft.typ");
    let ig = build_ignore(dir.path(), &file);
    // Both root and nested patterns should apply.
    assert!(ig.matched("test.bak", false).is_ignore());
    assert!(ig.matched("notes.draft.typ", false).is_ignore());
    assert!(!ig.matched("todo.typ", false).is_ignore());
}

// --- Watcher::new ---

#[test]
fn watcher_new_resolves_entries() {
    let (dir, dir_path) = setup_watch_dir();
    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let watcher = Watcher::new(store, &entries).unwrap();
    assert_eq!(watcher.entries.len(), 1);
    assert!(watcher.entries[0].path.is_absolute());
    drop(dir);
}

#[test]
fn watcher_new_bad_path_errors() {
    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![WatchEntry {
        path: "/nonexistent/path".to_string(),
        recursive: true,
    }];
    assert!(Watcher::new(store, &entries).is_err());
}

// --- initial_scan ---

#[test]
fn initial_scan_indexes_typ_files() {
    let (dir, dir_path) = setup_watch_dir();
    fs::write(
        dir_path.join("todo.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Buy milk\n",
    )
    .unwrap();
    fs::write(dir_path.join("notes.txt"), "not a typ file").unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    let result = watcher.initial_scan();

    assert_eq!(result.found, 1); // only .typ files
    assert_eq!(result.indexed, 1);
    assert_eq!(result.skipped, 0);
    assert_eq!(result.errors, 0);
    drop(dir);
}

#[test]
fn initial_scan_skips_unchanged() {
    let (dir, dir_path) = setup_watch_dir();
    fs::write(
        dir_path.join("todo.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Buy milk\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();

    let r1 = watcher.initial_scan();
    assert_eq!(r1.indexed, 1);

    let r2 = watcher.initial_scan();
    assert_eq!(r2.skipped, 1);
    assert_eq!(r2.indexed, 0);
    drop(dir);
}

#[test]
fn initial_scan_respects_mindtapeignore() {
    let (dir, dir_path) = setup_watch_dir();
    // Overwrite the default .mindtapeignore to also ignore ignored.typ.
    fs::write(dir_path.join(".mindtapeignore"), "lib/\nignored.typ\n").unwrap();
    fs::write(
        dir_path.join("todo.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Task\n",
    )
    .unwrap();
    fs::write(
        dir_path.join("ignored.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Hidden\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    let result = watcher.initial_scan();

    assert_eq!(result.found, 1); // ignored.typ filtered by walker
    drop(dir);
}

// --- handle_event ---

#[test]
fn handle_event_indexes_new_file() {
    let (dir, dir_path) = setup_watch_dir();
    let typ_file = dir_path.join("new.typ");
    fs::write(
        &typ_file,
        "#import \"@mindtape/mindtape:0.1.0\": id\n- [ ] New task #id(\"019c5b9b-7317-77b1-bf52-ce7a298cfcad\")\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    watcher.handle_event(&typ_file);

    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "New task");
    drop(dir);
}

#[test]
fn handle_event_removes_deleted_file() {
    let (dir, dir_path) = setup_watch_dir();
    let typ_file = dir_path.join("del.typ");
    fs::write(
        &typ_file,
        "#import \"@mindtape/mindtape:0.1.0\": id\n- [ ] Will be deleted #id(\"019c5b9b-7317-77b1-bf52-ce7a298cfcad\")\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();

    // Index it first.
    watcher.handle_event(&typ_file);
    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 1);

    // Delete the file, then handle the event.
    fs::remove_file(&typ_file).unwrap();
    watcher.handle_event(&typ_file);

    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 0);
    drop(dir);
}

#[test]
fn handle_event_ignores_non_typ() {
    let (dir, dir_path) = setup_watch_dir();
    let txt_file = dir_path.join("notes.txt");
    fs::write(&txt_file, "not typ").unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    watcher.handle_event(&txt_file);

    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 0);
    drop(dir);
}

#[test]
fn handle_event_ignores_mindtapeignored_file() {
    let (dir, dir_path) = setup_watch_dir();
    fs::write(dir_path.join(".mindtapeignore"), "secret.typ\n").unwrap();
    let typ_file = dir_path.join("secret.typ");
    fs::write(&typ_file, "- [ ] Secret task\n").unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    watcher.handle_event(&typ_file);

    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 0);
    drop(dir);
}

#[test]
fn handle_event_ignores_gitignored_file() {
    let (dir, dir_path) = setup_watch_dir();
    fs::write(dir_path.join(".gitignore"), "*.draft.typ\n").unwrap();
    let typ_file = dir_path.join("notes.draft.typ");
    fs::write(&typ_file, "- [ ] Draft task\n").unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    watcher.handle_event(&typ_file);

    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 0);
    drop(dir);
}

#[test]
fn initial_scan_respects_gitignore() {
    let (dir, dir_path) = setup_watch_dir();
    // WalkBuilder needs a .git dir to recognize .gitignore files.
    fs::create_dir(dir_path.join(".git")).unwrap();
    fs::write(dir_path.join(".gitignore"), "*.draft.typ\n").unwrap();
    fs::write(
        dir_path.join("todo.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Task\n",
    )
    .unwrap();
    fs::write(
        dir_path.join("notes.draft.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Hidden\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    let result = watcher.initial_scan();

    assert_eq!(result.found, 1); // .draft.typ filtered by walker
    assert_eq!(result.indexed, 1);
    drop(dir);
}

#[test]
fn handle_event_picks_up_new_gitignore() {
    let (dir, dir_path) = setup_watch_dir();
    let typ_file = dir_path.join("notes.draft.typ");
    fs::write(
        &typ_file,
        "#import \"@mindtape/mindtape:0.1.0\": id\n- [ ] Draft task #id(\"019c5b9b-7317-77b1-bf52-ce7a298cfcad\")\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();

    // Before .gitignore exists, file should be indexed.
    watcher.handle_event(&typ_file);
    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 1);

    // Create .gitignore after watcher started.
    fs::write(dir_path.join(".gitignore"), "*.draft.typ\n").unwrap();

    // Trigger the file again — now it should be ignored.
    // (The file won't be re-indexed because the ignore check happens first.)
    // Touch the file to force a hash change.
    fs::write(
        &typ_file,
        "#import \"@mindtape/mindtape:0.1.0\": id\n- [ ] Draft task updated #id(\"019c5b9b-7317-77b1-bf52-ce7a298cfcad\")\n",
    )
    .unwrap();
    watcher.handle_event(&typ_file);

    // The task from the first indexing should still be there,
    // but no new indexing should have occurred.
    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Draft task"); // not "Draft task updated"
    drop(dir);
}

#[test]
fn handle_event_ignores_deleted_non_typ_file() {
    let (dir, dir_path) = setup_watch_dir();
    let typ_file = dir_path.join("todo.typ");
    fs::write(
        &typ_file,
        "#import \"@mindtape/mindtape:0.1.0\": id\n- [ ] A task #id(\"019c5b9b-7317-77b1-bf52-ce7a298cfcad\")\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();

    // Index the .typ file first.
    watcher.handle_event(&typ_file);
    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 1);

    // Simulate a deleted non-typ file (e.g. a temp PDF).
    let pdf_tmp = dir_path.join("build/todo.pdf9wJeNG");
    // File doesn't exist — simulating post-deletion event.
    watcher.handle_event(&pdf_tmp);

    // The .typ task should still be there, unaffected.
    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 1);
    drop(dir);
}

#[test]
fn initial_scan_skips_non_mindtape_typ_files() {
    let (dir, dir_path) = setup_watch_dir();
    // File with mindtape import — should be indexed.
    fs::write(
        dir_path.join("tasks.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Real task\n",
    )
    .unwrap();
    // File without mindtape import — should be skipped silently.
    fs::write(dir_path.join("notes.typ"), "= Just notes\n\nSome text.\n").unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    let result = watcher.initial_scan();

    assert_eq!(result.found, 2); // both .typ files found
    assert_eq!(result.indexed, 1); // only the mindtape one indexed
    assert_eq!(result.skipped, 1); // the other skipped (no import)
    drop(dir);
}

#[test]
fn handle_event_unknown_path_is_noop() {
    let (dir, dir_path) = setup_watch_dir();
    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    // Path outside any watched entry.
    watcher.handle_event(Path::new("/tmp/other/file.typ"));
    drop(dir);
}

// --- resolve_entry ---

#[test]
fn resolve_entry_valid_path() {
    let dir = tempfile::tempdir().unwrap();
    let entry = make_entry(dir.path());
    let resolved = resolve_entry(&entry).unwrap();
    assert!(resolved.path.is_absolute());
}

#[test]
fn resolve_entry_bad_path_errors() {
    let entry = WatchEntry {
        path: "/nonexistent/path".to_string(),
        recursive: true,
    };
    assert!(resolve_entry(&entry).is_err());
}

// --- add_entry ---

#[test]
fn add_entry_new_path() {
    let (dir, dir_path) = setup_watch_dir();
    let sub = dir_path.join("sub");
    fs::create_dir(&sub).unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    assert_eq!(watcher.entries.len(), 1);

    let result = watcher.add_entry(&make_entry(&sub)).unwrap();
    assert!(result.is_some()); // (path, recursive)
    assert_eq!(watcher.entries.len(), 2);
    drop(dir);
}

#[test]
fn add_entry_duplicate_is_noop() {
    let (dir, dir_path) = setup_watch_dir();
    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();

    let result = watcher.add_entry(&make_entry(&dir_path)).unwrap();
    assert!(result.is_none());
    assert_eq!(watcher.entries.len(), 1);
    drop(dir);
}

// --- remove_entries_not_in ---

#[test]
fn remove_entries_not_in_removes_old() {
    let (dir, dir_path) = setup_watch_dir();
    let sub = dir_path.join("sub");
    fs::create_dir(&sub).unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path), make_entry(&sub)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    assert_eq!(watcher.entries.len(), 2);

    // Keep only the sub directory.
    let canon_sub = fs::canonicalize(&sub).unwrap();
    let keep: HashSet<PathBuf> = [canon_sub].into_iter().collect();
    let removed = watcher.remove_entries_not_in(&keep);

    assert_eq!(removed.len(), 1);
    assert_eq!(watcher.entries.len(), 1);
    assert_eq!(watcher.entries[0].path, fs::canonicalize(&sub).unwrap());
    drop(dir);
}

#[test]
fn remove_entries_not_in_keeps_matching() {
    let (dir, dir_path) = setup_watch_dir();
    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();

    let canon = fs::canonicalize(&dir_path).unwrap();
    let keep: HashSet<PathBuf> = [canon].into_iter().collect();
    let removed = watcher.remove_entries_not_in(&keep);

    assert!(removed.is_empty());
    assert_eq!(watcher.entries.len(), 1);
    drop(dir);
}

// --- scan_entries ---

#[test]
fn scan_entries_indexes_new_directory() {
    let (dir, dir_path) = setup_watch_dir();
    let sub = dir_path.join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(
        sub.join("tasks.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": due\n- [ ] Sub task\n",
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();

    let canon_sub = fs::canonicalize(&sub).unwrap();
    let result = watcher.scan_entries(&[canon_sub]);

    assert_eq!(result.found, 1);
    assert_eq!(result.indexed, 1);
    drop(dir);
}

// --- reload_config ---

#[test]
fn reload_config_adds_new_entry() {
    let (dir, dir_path) = setup_watch_dir();
    let sub = dir_path.join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(
        sub.join("tasks.typ"),
        "#import \"@mindtape/mindtape:0.1.0\": id\n- [ ] Sub task #id(\"019c5b9b-7317-77b1-bf52-ce7a298cfcad\")\n",
    )
    .unwrap();

    // Write initial config with just the root.
    let config_path = dir_path.join("mindtape.toml");
    fs::write(
        &config_path,
        format!("[[watch]]\npath = \"{}\"\n", dir_path.display()),
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    assert_eq!(watcher.entries.len(), 1);

    // Update config to add the sub directory.
    fs::write(
        &config_path,
        format!(
            "[[watch]]\npath = \"{}\"\n\n[[watch]]\npath = \"{}\"\n",
            dir_path.display(),
            sub.display()
        ),
    )
    .unwrap();

    // Create a notify watcher (we won't use its events, just need it for API).
    let (tx, _rx) = std::sync::mpsc::channel();
    let mut notify_watcher: notify::RecommendedWatcher =
        notify::Watcher::new(tx, notify::Config::default()).unwrap();
    notify_watcher
        .watch(&dir_path, notify::RecursiveMode::Recursive)
        .unwrap();

    watcher.reload_config(&[config_path], &mut notify_watcher);

    assert_eq!(watcher.entries.len(), 2);
    // The sub directory's tasks should have been scanned.
    let tasks = watcher
        .store
        .query_tasks(&crate::store::TaskFilter::default())
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Sub task");
    drop(dir);
}

#[test]
fn reload_config_removes_old_entry() {
    let (dir, dir_path) = setup_watch_dir();
    let sub = dir_path.join("sub");
    fs::create_dir(&sub).unwrap();

    // Start with both directories watched.
    let config_path = dir_path.join("mindtape.toml");
    fs::write(
        &config_path,
        format!(
            "[[watch]]\npath = \"{}\"\n\n[[watch]]\npath = \"{}\"\n",
            dir_path.display(),
            sub.display()
        ),
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path), make_entry(&sub)];
    let mut watcher = Watcher::new(store, &entries).unwrap();
    assert_eq!(watcher.entries.len(), 2);

    // Update config to remove the sub directory.
    fs::write(
        &config_path,
        format!("[[watch]]\npath = \"{}\"\n", dir_path.display()),
    )
    .unwrap();

    let (tx, _rx) = std::sync::mpsc::channel();
    let mut notify_watcher: notify::RecommendedWatcher =
        notify::Watcher::new(tx, notify::Config::default()).unwrap();
    notify_watcher
        .watch(&dir_path, notify::RecursiveMode::Recursive)
        .unwrap();
    notify_watcher
        .watch(&sub, notify::RecursiveMode::Recursive)
        .unwrap();

    watcher.reload_config(&[config_path], &mut notify_watcher);

    assert_eq!(watcher.entries.len(), 1);
    assert_eq!(
        watcher.entries[0].path,
        fs::canonicalize(&dir_path).unwrap()
    );
    drop(dir);
}

#[test]
fn reload_config_invalid_toml_is_nonfatal() {
    let (dir, dir_path) = setup_watch_dir();
    let config_path = dir_path.join("mindtape.toml");
    fs::write(
        &config_path,
        format!("[[watch]]\npath = \"{}\"\n", dir_path.display()),
    )
    .unwrap();

    let store = SqliteStore::open_memory().unwrap();
    let entries = vec![make_entry(&dir_path)];
    let mut watcher = Watcher::new(store, &entries).unwrap();

    // Write garbage config.
    fs::write(&config_path, "not valid [[[toml").unwrap();

    let (tx, _rx) = std::sync::mpsc::channel();
    let mut notify_watcher: notify::RecommendedWatcher =
        notify::Watcher::new(tx, notify::Config::default()).unwrap();

    // Should not panic or error — just log a warning.
    watcher.reload_config(&[config_path], &mut notify_watcher);

    // Entries should be unchanged.
    assert_eq!(watcher.entries.len(), 1);
    drop(dir);
}
